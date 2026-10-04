#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

use std::{
    fs::OpenOptions,
    io::Write,
    ptr::read_unaligned,
    thread,
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, SYSTEMTIME},
    System::{
        Memory::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ},
        SystemInformation::GetLocalTime,
    },
};

const MAP_NAME: &str = "LMU_Data";
const MAX_VEHICLES: usize = 104;

const OFF_SCORING_INFO: usize = 1632;
const OFF_SESSION: usize = OFF_SCORING_INFO + 64;
const OFF_LAP_LENGTH: usize = OFF_SCORING_INFO + 88;
const OFF_NUM_VEHICLES: usize = OFF_SCORING_INFO + 104;
const OFF_VEH_SCORING: usize = 2192;
const SCORING_STRIDE: usize = 584;

const SC_ID: usize = 0;
const SC_DRIVER_NAME: usize = 4;
const SC_TOTAL_LAPS: usize = 100;
const SC_FINISH_STATUS: usize = 103;
const SC_LAP_DIST: usize = 104;
const SC_PATH_LATERAL: usize = 112;
const SC_TRACK_EDGE: usize = 120;
const SC_IS_PLAYER: usize = 196;
const SC_IN_PITS: usize = 198;
const SC_PLACE: usize = 199;

const OFF_ACTIVE_VEHICLES: usize = 128_464;
const OFF_PLAYER_IDX: usize = OFF_ACTIVE_VEHICLES + 1;
const OFF_PLAYER_HAS_VEHICLE: usize = OFF_ACTIVE_VEHICLES + 2;
const OFF_TELEM_INFO: usize = 128_468;
const TELEMETRY_STRIDE: usize = 1888;

const T_LOCAL_VEL: usize = 184;
const T_LAST_IMPACT_ET: usize = 552;
const T_LAST_IMPACT_MAG: usize = 560;
const T_GAP_AHEAD: usize = 780;
const T_GAP_BEHIND: usize = 784;
const T_WHEELS: usize = 848;
const WHEEL_STRIDE: usize = 260;
const W_SURFACE_TYPE: usize = 176;

const SAMPLE_PERIOD: Duration = Duration::from_millis(100);
const POSITION_CONFIRM: Duration = Duration::from_secs(2);
const BATTLE_MIN: Duration = Duration::from_secs(5);
const BATTLE_COOLDOWN: Duration = Duration::from_secs(20);
const INCIDENT_QUIET: Duration = Duration::from_secs(3);
const ADJACENT_DISTANCE_M: f64 = 400.0;
const CONTACT_RECORD_MIN: f64 = 80.0;
const CONTACT_HIGHLIGHT_MIN: f64 = 500.0;
const POSITION_EVENT_DEDUP: Duration = Duration::from_secs(8);

struct Mapping {
    handle: HANDLE,
    ptr: *const u8,
}

impl Mapping {
    fn open() -> Option<Self> {
        let mut wide: Vec<u16> = MAP_NAME.encode_utf16().collect();
        wide.push(0);
        unsafe {
            let handle = OpenFileMappingW(FILE_MAP_READ, 0, wide.as_ptr());
            if handle.is_null() {
                return None;
            }
            let view = MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 0);
            if view.Value.is_null() {
                CloseHandle(handle);
                return None;
            }
            Some(Self { handle, ptr: view.Value as *const u8 })
        }
    }

    #[inline]
    fn read<T: Copy>(&self, offset: usize) -> T {
        unsafe { read_unaligned(self.ptr.add(offset) as *const T) }
    }

    fn read_name(&self, offset: usize, len: usize) -> String {
        let bytes = unsafe { std::slice::from_raw_parts(self.ptr.add(offset), len) };
        let end = bytes.iter().position(|&b| b == 0).unwrap_or(len);
        String::from_utf8_lossy(&bytes[..end]).trim().to_string()
    }
}

impl Drop for Mapping {
    fn drop(&mut self) {
        unsafe {
            let _ = UnmapViewOfFile(windows_sys::Win32::System::Memory::MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.ptr as *mut _,
            });
            CloseHandle(self.handle);
        }
    }
}

#[derive(Clone)]
struct RivalSnapshot {
    id: i32,
    name: String,
    place: u8,
    in_pits: bool,
    distance: f64,
}

struct PendingPosition {
    old: u8,
    new: u8,
    started: Instant,
    rival: RivalSnapshot,
    session: i32,
    lap_dist: f64,
}

struct LastPositionEvent {
    rival_id: i32,
    overtake: bool,
    at: Instant,
}

struct Incident {
    started: Instant,
    last_signal: Instant,
    contacts: u32,
    max_impact: f64,
    offtrack: bool,
    max_wheels_off: usize,
    spin: bool,
    max_speed_kmh: f64,
}

impl Incident {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            last_signal: now,
            contacts: 0,
            max_impact: 0.0,
            offtrack: false,
            max_wheels_off: 0,
            spin: false,
            max_speed_kmh: 0.0,
        }
    }

    fn touch(&mut self, speed_kmh: f64) {
        self.last_signal = Instant::now();
        self.max_speed_kmh = self.max_speed_kmh.max(speed_kmh);
    }

    fn priority(&self) -> Option<&'static str> {
        if self.spin || self.max_impact >= 2000.0 || (self.offtrack && self.max_speed_kmh >= 180.0) {
            Some("HIGH")
        } else if self.offtrack || self.max_impact >= CONTACT_HIGHLIGHT_MIN {
            Some("MEDIUM")
        } else {
            None
        }
    }
}

struct State {
    scoring_idx: Option<usize>,
    last_place: u8,
    last_finish: i8,
    last_impact_et: Option<f64>,
    last_ahead: Option<RivalSnapshot>,
    last_behind: Option<RivalSnapshot>,
    close_ahead_before: bool,
    close_behind_before: bool,
    pending_position: Option<PendingPosition>,
    last_position_event: Option<LastPositionEvent>,
    offtrack_since: Option<Instant>,
    offtrack_active: bool,
    spin_since: Option<Instant>,
    spin_active: bool,
    battle_since: Option<Instant>,
    battle_active: bool,
    last_battle_emit: Option<Instant>,
    incident: Option<Incident>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            scoring_idx: None,
            last_place: 0,
            last_finish: 0,
            last_impact_et: None,
            last_ahead: None,
            last_behind: None,
            close_ahead_before: false,
            close_behind_before: false,
            pending_position: None,
            last_position_event: None,
            offtrack_since: None,
            offtrack_active: false,
            spin_since: None,
            spin_active: false,
            battle_since: None,
            battle_active: false,
            last_battle_emit: None,
            incident: None,
        }
    }
}

fn now_string() -> String {
    unsafe {
        let mut t: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut t);
        format!("{:02}:{:02}:{:02}", t.wHour, t.wMinute, t.wSecond)
    }
}

fn emit(log: &mut std::fs::File, kind: &str, details: &str) {
    let line = format!("{} | {:<28} | {}\n", now_string(), kind, details);
    print!("{line}");
    let _ = log.write_all(line.as_bytes());
    let _ = log.flush();
}

fn player_scoring_index(map: &Mapping, cached: Option<usize>) -> Option<usize> {
    if let Some(i) = cached {
        if i < MAX_VEHICLES {
            let base = OFF_VEH_SCORING + i * SCORING_STRIDE;
            if map.read::<u8>(base + SC_IS_PLAYER) != 0 {
                return Some(i);
            }
        }
    }
    let n = map.read::<i32>(OFF_NUM_VEHICLES).clamp(0, MAX_VEHICLES as i32) as usize;
    (0..n).find(|&i| map.read::<u8>(OFF_VEH_SCORING + i * SCORING_STRIDE + SC_IS_PLAYER) != 0)
}

fn find_place(map: &Mapping, place: u8) -> Option<usize> {
    if place == 0 {
        return None;
    }
    let n = map.read::<i32>(OFF_NUM_VEHICLES).clamp(0, MAX_VEHICLES as i32) as usize;
    (0..n).find(|&i| {
        let base = OFF_VEH_SCORING + i * SCORING_STRIDE;
        map.read::<u8>(base + SC_PLACE) == place
    })
}

fn find_vehicle_id(map: &Mapping, id: i32) -> Option<usize> {
    let n = map.read::<i32>(OFF_NUM_VEHICLES).clamp(0, MAX_VEHICLES as i32) as usize;
    (0..n).find(|&i| map.read::<i32>(OFF_VEH_SCORING + i * SCORING_STRIDE + SC_ID) == id)
}

fn race_distance(map: &Mapping, scoring_idx: usize, lap_len: f64) -> f64 {
    let base = OFF_VEH_SCORING + scoring_idx * SCORING_STRIDE;
    let laps = map.read::<i16>(base + SC_TOTAL_LAPS).max(0) as f64;
    let dist = map.read::<f64>(base + SC_LAP_DIST);
    if lap_len.is_finite() && lap_len > 100.0 {
        laps * lap_len + dist
    } else {
        dist
    }
}

fn rival_at_place(map: &Mapping, place: u8, player_idx: usize, lap_len: f64) -> Option<RivalSnapshot> {
    let idx = find_place(map, place)?;
    if idx == player_idx {
        return None;
    }
    let base = OFF_VEH_SCORING + idx * SCORING_STRIDE;
    let id = map.read::<i32>(base + SC_ID);
    let name = map.read_name(base + SC_DRIVER_NAME, 32);
    let in_pits = map.read::<u8>(base + SC_IN_PITS) != 0;
    let player_distance = race_distance(map, player_idx, lap_len);
    let rival_distance = race_distance(map, idx, lap_len);
    Some(RivalSnapshot {
        id,
        name,
        place,
        in_pits,
        distance: (player_distance - rival_distance).abs(),
    })
}

fn valid_gap(g: f32) -> Option<f32> {
    if g.is_finite() && g > 0.0 && g <= 20.0 { Some(g) } else { None }
}

fn fmt_gap(g: Option<f32>) -> String {
    g.map(|v| format!("{v:.2}s")).unwrap_or_else(|| "n/a".to_string())
}

fn incident_mut(st: &mut State) -> &mut Incident {
    st.incident.get_or_insert_with(Incident::new)
}

fn maybe_flush_incident(st: &mut State, log: &mut std::fs::File) {
    let ready = st.incident.as_ref()
        .map(|i| i.last_signal.elapsed() >= INCIDENT_QUIET)
        .unwrap_or(false);
    if !ready { return; }

    if let Some(i) = st.incident.take() {
        let details = format!(
            "contacts={} | maxImpact={:.0} | offTrack={} | wheelsOff={} | spin={} | maxSpeed={:.0}km/h | {:.1}s",
            i.contacts,
            i.max_impact,
            if i.offtrack { "yes" } else { "no" },
            i.max_wheels_off,
            if i.spin { "yes" } else { "no" },
            i.max_speed_kmh,
            i.started.elapsed().as_secs_f32()
        );

        if let Some(priority) = i.priority() {
            emit(log, &format!("INCIDENT_{}", priority), &details);
        } else {
            // Diagnostic only: this will NOT be eligible to trigger AutoClips.
            emit(log, "CONTACT_FILTERED", &format!("{} | no highlight", details));
        }
    }
}

fn main() {
    println!("============================================");
    println!("       RISAN TELEMETRY BRIDGE v0.5");
    println!("============================================");
    println!("Diagnostico LMU: no crea clips todavia.");
    println!("Adelantamientos confirmados + deduplicacion + filtro de contactos.");
    println!("CPU objetivo: minimo | GPU: 0 | Internet: 0");
    println!("Log: RisanTelemetryEvents.log\n");

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("RisanTelemetryEvents.log")
        .expect("No se pudo abrir el log");

    emit(&mut log, "BRIDGE_START", "v0.5 | Esperando LMU_Data");

    loop {
        let Some(map) = Mapping::open() else {
            print!("\rEsperando a Le Mans Ultimate...          ");
            let _ = std::io::stdout().flush();
            thread::sleep(Duration::from_secs(2));
            continue;
        };

        println!("\nLMU_Data detectado. Telemetria activa.");
        emit(&mut log, "LMU_CONNECTED", "Shared memory LMU_Data abierta");

        let mut st = State::default();

        loop {
            let active = map.read::<u8>(OFF_ACTIVE_VEHICLES) as usize;
            let has_player = map.read::<u8>(OFF_PLAYER_HAS_VEHICLE) != 0;
            let t_idx = map.read::<u8>(OFF_PLAYER_IDX) as usize;
            st.scoring_idx = player_scoring_index(&map, st.scoring_idx);

            if !has_player || t_idx >= MAX_VEHICLES || active == 0 {
                maybe_flush_incident(&mut st, &mut log);
                thread::sleep(Duration::from_millis(500));
                if Mapping::open().is_none() {
                    emit(&mut log, "LMU_DISCONNECTED", "Esperando siguiente sesion");
                    break;
                }
                continue;
            }

            let Some(s_idx) = st.scoring_idx else {
                thread::sleep(Duration::from_millis(250));
                continue;
            };

            let sbase = OFF_VEH_SCORING + s_idx * SCORING_STRIDE;
            let tbase = OFF_TELEM_INFO + t_idx * TELEMETRY_STRIDE;

            let session = map.read::<i32>(OFF_SESSION);
            let lap_len = map.read::<f64>(OFF_LAP_LENGTH);
            let place = map.read::<u8>(sbase + SC_PLACE);
            let finish = map.read::<i8>(sbase + SC_FINISH_STATUS);
            let in_pits = map.read::<u8>(sbase + SC_IN_PITS) != 0;
            let lap_dist = map.read::<f64>(sbase + SC_LAP_DIST);
            let path_lat = map.read::<f64>(sbase + SC_PATH_LATERAL);
            let track_edge = map.read::<f64>(sbase + SC_TRACK_EDGE);

            if st.last_place != 0 && place != 0 && place != st.last_place {
                let delta = place as i16 - st.last_place as i16;
                let rival = if delta == -1 {
                    st.last_ahead.clone()
                } else if delta == 1 {
                    st.last_behind.clone()
                } else {
                    None
                };

                if delta.abs() == 1 && !in_pits {
                    if let Some(rival) = rival {
                        if !rival.in_pits {
                            st.pending_position = Some(PendingPosition {
                                old: st.last_place,
                                new: place,
                                started: Instant::now(),
                                rival,
                                session,
                                lap_dist,
                            });
                        } else {
                            emit(&mut log, "POSITION_CHANGE", &format!(
                                "P{} -> P{} | rival={} in pits | no highlight",
                                st.last_place, place, rival.name
                            ));
                            st.pending_position = None;
                        }
                    } else {
                        emit(&mut log, "POSITION_CHANGE", &format!(
                            "P{} -> P{} | rival identity unavailable | no highlight",
                            st.last_place, place
                        ));
                        st.pending_position = None;
                    }
                } else {
                    emit(&mut log, "POSITION_CHANGE", &format!(
                        "P{} -> P{} | delta={} | pits={} | no highlight",
                        st.last_place, place, delta, in_pits
                    ));
                    st.pending_position = None;
                }
            }
            st.last_place = place;

            if let Some(p) = &st.pending_position {
                if place != p.new {
                    st.pending_position = None;
                } else if p.started.elapsed() >= POSITION_CONFIRM {
                    let rival_now = find_vehicle_id(&map, p.rival.id).map(|idx| {
                        let base = OFF_VEH_SCORING + idx * SCORING_STRIDE;
                        (
                            map.read::<u8>(base + SC_PLACE),
                            map.read::<u8>(base + SC_IN_PITS) != 0,
                        )
                    });

                    match rival_now {
                        Some((rival_place, rival_in_pits)) if !rival_in_pits && rival_place == p.old => {
                            let overtake = p.new < p.old;
                            let duplicate = st.last_position_event.as_ref()
                                .map(|e| e.rival_id == p.rival.id
                                    && e.overtake == overtake
                                    && e.at.elapsed() < POSITION_EVENT_DEDUP)
                                .unwrap_or(false);

                            if duplicate {
                                emit(&mut log, "POSITION_DUPLICATE_FILTERED", &format!(
                                    "P{} -> P{} | rival={} (id={}) | same event within {}s | no highlight",
                                    p.old, p.new, p.rival.name, p.rival.id, POSITION_EVENT_DEDUP.as_secs()
                                ));
                            } else {
                                let kind = if overtake {
                                    "OVERTAKE_CONFIRMED"
                                } else {
                                    "POSITION_LOSS_CONFIRMED"
                                };
                                emit(&mut log, kind, &format!(
                                    "P{} -> P{} | rival={} (id={}) swapped to P{} | stable=2s | preDistance={:.0}m | lapDist={:.0}m | session={}",
                                    p.old, p.new, p.rival.name, p.rival.id, rival_place, p.rival.distance, p.lap_dist, p.session
                                ));
                                st.last_position_event = Some(LastPositionEvent {
                                    rival_id: p.rival.id,
                                    overtake,
                                    at: Instant::now(),
                                });
                            }
                            st.pending_position = None;
                        }
                        Some((rival_place, rival_in_pits)) => {
                            emit(&mut log, "POSITION_CHANGE", &format!(
                                "P{} -> P{} | rival={} now P{} pits={} | swap not confirmed",
                                p.old, p.new, p.rival.name, rival_place, rival_in_pits
                            ));
                            st.pending_position = None;
                        }
                        None => {
                            emit(&mut log, "POSITION_CHANGE", &format!(
                                "P{} -> P{} | rival={} disappeared | no highlight",
                                p.old, p.new, p.rival.name
                            ));
                            st.pending_position = None;
                        }
                    }
                }
            }

            if finish == 1 && st.last_finish != 1 {
                emit(&mut log, "RACE_FINISH", &format!("Final P{} | session={}", place, session));
            }
            st.last_finish = finish;

            let vx = map.read::<f64>(tbase + T_LOCAL_VEL);
            let vy = map.read::<f64>(tbase + T_LOCAL_VEL + 8);
            let vz = map.read::<f64>(tbase + T_LOCAL_VEL + 16);
            let speed = (vx * vx + vy * vy + vz * vz).sqrt();
            let speed_kmh = speed * 3.6;

            let impact_et = map.read::<f64>(tbase + T_LAST_IMPACT_ET);
            let impact_mag = map.read::<f64>(tbase + T_LAST_IMPACT_MAG);
            match st.last_impact_et {
                None => st.last_impact_et = Some(impact_et),
                Some(prev) if impact_et.is_finite() && impact_et > 0.0 && impact_et > prev + 0.001 => {
                    // Always advance the LMU impact clock, but ignore tiny bumps/kerb noise.
                    st.last_impact_et = Some(impact_et);
                    if impact_mag.is_finite() && impact_mag >= CONTACT_RECORD_MIN {
                        let inc = incident_mut(&mut st);
                        inc.contacts += 1;
                        inc.max_impact = inc.max_impact.max(impact_mag);
                        inc.touch(speed_kmh);
                    }
                }
                _ => {}
            }

            let mut bad_surface = 0usize;
            for wheel in 0..4 {
                let surface = map.read::<u8>(tbase + T_WHEELS + wheel * WHEEL_STRIDE + W_SURFACE_TYPE);
                if matches!(surface, 2 | 3 | 4) { bad_surface += 1; }
            }

            let offtrack = speed_kmh > 20.0
                && (bad_surface >= 2 || (track_edge > 0.0 && path_lat.abs() > track_edge + 0.5));

            if offtrack {
                let since = st.offtrack_since.get_or_insert_with(Instant::now);
                if !st.offtrack_active && since.elapsed() >= Duration::from_millis(450) {
                    let inc = incident_mut(&mut st);
                    inc.offtrack = true;
                    inc.max_wheels_off = inc.max_wheels_off.max(bad_surface);
                    inc.touch(speed_kmh);
                    st.offtrack_active = true;
                }
            } else {
                st.offtrack_since = None;
                st.offtrack_active = false;
            }

            let forward = vz.abs();
            let lateral = vx.abs();
            let possible_spin = speed_kmh > 45.0 && lateral > 6.0 && lateral > forward * 0.35;
            if possible_spin {
                let since = st.spin_since.get_or_insert_with(Instant::now);
                if !st.spin_active && since.elapsed() >= Duration::from_millis(550) {
                    let inc = incident_mut(&mut st);
                    inc.spin = true;
                    inc.touch(speed_kmh);
                    st.spin_active = true;
                }
            } else {
                st.spin_since = None;
                st.spin_active = false;
            }

            maybe_flush_incident(&mut st, &mut log);

            let gap_a = valid_gap(map.read::<f32>(tbase + T_GAP_AHEAD));
            let gap_b = valid_gap(map.read::<f32>(tbase + T_GAP_BEHIND));

            let current_ahead = if place > 1 { rival_at_place(&map, place - 1, s_idx, lap_len) } else { None };
            let current_behind = rival_at_place(&map, place.saturating_add(1), s_idx, lap_len);

            st.close_ahead_before = gap_a.map(|g| g <= 1.5).unwrap_or(false)
                || current_ahead.as_ref().map(|r| r.distance <= ADJACENT_DISTANCE_M).unwrap_or(false);
            st.close_behind_before = gap_b.map(|g| g <= 1.5).unwrap_or(false)
                || current_behind.as_ref().map(|r| r.distance <= ADJACENT_DISTANCE_M).unwrap_or(false);

            let battle_close = !in_pits
                && speed_kmh > 40.0
                && (st.close_ahead_before || st.close_behind_before);

            if battle_close {
                let since = st.battle_since.get_or_insert_with(Instant::now);
                let cooldown_ok = st.last_battle_emit.map(|t| t.elapsed() >= BATTLE_COOLDOWN).unwrap_or(true);
                if !st.battle_active && cooldown_ok && since.elapsed() >= BATTLE_MIN {
                    emit(&mut log, "CLOSE_BATTLE", &format!(
                        "ahead={} | behind={} | aheadRival={} | behindRival={}",
                        fmt_gap(gap_a),
                        fmt_gap(gap_b),
                        current_ahead.as_ref().map(|r| r.name.as_str()).unwrap_or("n/a"),
                        current_behind.as_ref().map(|r| r.name.as_str()).unwrap_or("n/a")
                    ));
                    st.battle_active = true;
                    st.last_battle_emit = Some(Instant::now());
                }
            } else {
                st.battle_since = None;
                st.battle_active = false;
            }

            // Store adjacent rival identities for the NEXT sample. A real pass is
            // confirmed when one of these same IDs swaps places with the player.
            st.last_ahead = current_ahead;
            st.last_behind = current_behind;

            thread::sleep(SAMPLE_PERIOD);
        }

        thread::sleep(Duration::from_secs(1));
    }
}
