use std::{
    ptr::read_unaligned,
    time::{Duration, Instant},
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ},
};

pub const BRIDGE_SCHEMA_VERSION: &str = "1.0";
pub const SOURCE_NAME: &str = "LMU_Data";

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

const POSITION_CONFIRM: Duration = Duration::from_secs(2);
const POSITION_EVENT_DEDUP: Duration = Duration::from_secs(8);
const BATTLE_MIN: Duration = Duration::from_secs(5);
const BATTLE_COOLDOWN: Duration = Duration::from_secs(20);
const INCIDENT_QUIET: Duration = Duration::from_secs(3);
const ADJACENT_DISTANCE_M: f64 = 400.0;
const CONTACT_RECORD_MIN: f64 = 80.0;
const CONTACT_HIGHLIGHT_MIN: f64 = 500.0;

#[derive(Clone, Debug)]
pub struct BridgeEvent {
    pub schema_version: &'static str,
    pub kind: &'static str,
    pub details: String,
    pub highlight_eligible: bool,
    pub priority: u8,
}

#[derive(Clone, Debug, Default)]
pub struct BridgeSnapshot {
    pub session: i32,
    pub place: u8,
    pub in_pits: bool,
    pub lap_dist: f64,
    pub speed_kmh: f64,
    pub gap_ahead_s: Option<f32>,
    pub gap_behind_s: Option<f32>,
}

pub struct PollResult {
    pub active: bool,
    pub snapshot: Option<BridgeSnapshot>,
    pub events: Vec<BridgeEvent>,
}

struct Mapping {
    handle: HANDLE,
    ptr: *const u8,
}

impl Mapping {
    fn open() -> Option<Self> {
        let mut wide: Vec<u16> = SOURCE_NAME.encode_utf16().collect();
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

    fn priority(&self) -> Option<(&'static str, u8)> {
        if self.spin || self.max_impact >= 2000.0 || (self.offtrack && self.max_speed_kmh >= 180.0) {
            Some(("INCIDENT_HIGH", 100))
        } else if self.offtrack || self.max_impact >= CONTACT_HIGHLIGHT_MIN {
            Some(("INCIDENT_MEDIUM", 75))
        } else {
            None
        }
    }
}

#[derive(Default)]
struct State {
    scoring_idx: Option<usize>,
    last_place: u8,
    last_finish: i8,
    last_impact_et: Option<f64>,
    last_ahead: Option<RivalSnapshot>,
    last_behind: Option<RivalSnapshot>,
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

pub struct LmuDataBridgeCore {
    map: Mapping,
    st: State,
}

impl LmuDataBridgeCore {
    pub fn source_available() -> bool {
        Mapping::open().is_some()
    }

    pub fn connect() -> Option<Self> {
        Some(Self {
            map: Mapping::open()?,
            st: State::default(),
        })
    }

    pub fn poll(&mut self) -> PollResult {
        let mut events = Vec::new();

        let active = self.map.read::<u8>(OFF_ACTIVE_VEHICLES) as usize;
        let has_player = self.map.read::<u8>(OFF_PLAYER_HAS_VEHICLE) != 0;
        let t_idx = self.map.read::<u8>(OFF_PLAYER_IDX) as usize;
        self.st.scoring_idx = player_scoring_index(&self.map, self.st.scoring_idx);

        if !has_player || t_idx >= MAX_VEHICLES || active == 0 {
            flush_incident(&mut self.st, &mut events);
            return PollResult { active: false, snapshot: None, events };
        }

        let Some(s_idx) = self.st.scoring_idx else {
            return PollResult { active: false, snapshot: None, events };
        };

        let sbase = OFF_VEH_SCORING + s_idx * SCORING_STRIDE;
        let tbase = OFF_TELEM_INFO + t_idx * TELEMETRY_STRIDE;

        let session = self.map.read::<i32>(OFF_SESSION);
        let lap_len = self.map.read::<f64>(OFF_LAP_LENGTH);
        let place = self.map.read::<u8>(sbase + SC_PLACE);
        let finish = self.map.read::<i8>(sbase + SC_FINISH_STATUS);
        let in_pits = self.map.read::<u8>(sbase + SC_IN_PITS) != 0;
        let lap_dist = self.map.read::<f64>(sbase + SC_LAP_DIST);
        let path_lat = self.map.read::<f64>(sbase + SC_PATH_LATERAL);
        let track_edge = self.map.read::<f64>(sbase + SC_TRACK_EDGE);

        let vx = self.map.read::<f64>(tbase + T_LOCAL_VEL);
        let vy = self.map.read::<f64>(tbase + T_LOCAL_VEL + 8);
        let vz = self.map.read::<f64>(tbase + T_LOCAL_VEL + 16);
        let speed_kmh = (vx * vx + vy * vy + vz * vz).sqrt() * 3.6;

        if self.st.last_place != 0 && place != 0 && place != self.st.last_place {
            let delta = place as i16 - self.st.last_place as i16;
            let rival = if delta == -1 {
                self.st.last_ahead.clone()
            } else if delta == 1 {
                self.st.last_behind.clone()
            } else {
                None
            };

            if delta.abs() == 1 && !in_pits {
                if let Some(rival) = rival {
                    if !rival.in_pits {
                        self.st.pending_position = Some(PendingPosition {
                            old: self.st.last_place,
                            new: place,
                            started: Instant::now(),
                            rival,
                            session,
                            lap_dist,
                        });
                    } else {
                        push_event(&mut events, "POSITION_CHANGE", false, 0, format!(
                            "P{} -> P{} | rival={} in pits | no highlight",
                            self.st.last_place, place, rival.name
                        ));
                        self.st.pending_position = None;
                    }
                } else {
                    push_event(&mut events, "POSITION_CHANGE", false, 0, format!(
                        "P{} -> P{} | rival identity unavailable | no highlight",
                        self.st.last_place, place
                    ));
                    self.st.pending_position = None;
                }
            } else {
                push_event(&mut events, "POSITION_CHANGE", false, 0, format!(
                    "P{} -> P{} | delta={} | pits={} | no highlight",
                    self.st.last_place, place, delta, in_pits
                ));
                self.st.pending_position = None;
            }
        }
        self.st.last_place = place;

        if let Some(p) = &self.st.pending_position {
            if place != p.new {
                self.st.pending_position = None;
            } else if p.started.elapsed() >= POSITION_CONFIRM {
                let rival_now = find_vehicle_id(&self.map, p.rival.id).map(|idx| {
                    let base = OFF_VEH_SCORING + idx * SCORING_STRIDE;
                    (
                        self.map.read::<u8>(base + SC_PLACE),
                        self.map.read::<u8>(base + SC_IN_PITS) != 0,
                    )
                });

                match rival_now {
                    Some((rival_place, rival_in_pits)) if !rival_in_pits && rival_place == p.old => {
                        let overtake = p.new < p.old;
                        let duplicate = self.st.last_position_event.as_ref()
                            .map(|e| e.rival_id == p.rival.id
                                && e.overtake == overtake
                                && e.at.elapsed() < POSITION_EVENT_DEDUP)
                            .unwrap_or(false);

                        if duplicate {
                            push_event(&mut events, "POSITION_DUPLICATE_FILTERED", false, 0, format!(
                                "P{} -> P{} | rival={} (id={}) | same event within {}s | no highlight",
                                p.old, p.new, p.rival.name, p.rival.id, POSITION_EVENT_DEDUP.as_secs()
                            ));
                        } else {
                            let kind = if overtake { "OVERTAKE_CONFIRMED" } else { "POSITION_LOSS_CONFIRMED" };
                            push_event(&mut events, kind, true, 85, format!(
                                "P{} -> P{} | rival={} (id={}) swapped to P{} | stable=2s | preDistance={:.0}m | lapDist={:.0}m | session={}",
                                p.old, p.new, p.rival.name, p.rival.id, rival_place, p.rival.distance, p.lap_dist, p.session
                            ));
                            self.st.last_position_event = Some(LastPositionEvent {
                                rival_id: p.rival.id,
                                overtake,
                                at: Instant::now(),
                            });
                        }
                        self.st.pending_position = None;
                    }
                    Some((rival_place, rival_in_pits)) => {
                        push_event(&mut events, "POSITION_CHANGE", false, 0, format!(
                            "P{} -> P{} | rival={} now P{} pits={} | swap not confirmed",
                            p.old, p.new, p.rival.name, rival_place, rival_in_pits
                        ));
                        self.st.pending_position = None;
                    }
                    None => {
                        push_event(&mut events, "POSITION_CHANGE", false, 0, format!(
                            "P{} -> P{} | rival={} disappeared | no highlight",
                            p.old, p.new, p.rival.name
                        ));
                        self.st.pending_position = None;
                    }
                }
            }
        }

        if finish == 1 && self.st.last_finish != 1 {
            push_event(&mut events, "RACE_FINISH", true, 90, format!("Final P{} | session={}", place, session));
        }
        self.st.last_finish = finish;

        let impact_et = self.map.read::<f64>(tbase + T_LAST_IMPACT_ET);
        let impact_mag = self.map.read::<f64>(tbase + T_LAST_IMPACT_MAG);
        match self.st.last_impact_et {
            None => self.st.last_impact_et = Some(impact_et),
            Some(prev) if impact_et.is_finite() && impact_et > 0.0 && impact_et > prev + 0.001 => {
                self.st.last_impact_et = Some(impact_et);
                if impact_mag.is_finite() && impact_mag >= CONTACT_RECORD_MIN {
                    let inc = self.st.incident.get_or_insert_with(Incident::new);
                    inc.contacts += 1;
                    inc.max_impact = inc.max_impact.max(impact_mag);
                    inc.touch(speed_kmh);
                }
            }
            _ => {}
        }

        let mut bad_surface = 0usize;
        for wheel in 0..4 {
            let surface = self.map.read::<u8>(tbase + T_WHEELS + wheel * WHEEL_STRIDE + W_SURFACE_TYPE);
            if matches!(surface, 2 | 3 | 4) { bad_surface += 1; }
        }

        let offtrack = speed_kmh > 20.0
            && (bad_surface >= 2 || (track_edge > 0.0 && path_lat.abs() > track_edge + 0.5));

        if offtrack {
            let since = self.st.offtrack_since.get_or_insert_with(Instant::now);
            if !self.st.offtrack_active && since.elapsed() >= Duration::from_millis(450) {
                let inc = self.st.incident.get_or_insert_with(Incident::new);
                inc.offtrack = true;
                inc.max_wheels_off = inc.max_wheels_off.max(bad_surface);
                inc.touch(speed_kmh);
                self.st.offtrack_active = true;
            }
        } else {
            self.st.offtrack_since = None;
            self.st.offtrack_active = false;
        }

        let forward = vz.abs();
        let lateral = vx.abs();
        let possible_spin = speed_kmh > 45.0 && lateral > 6.0 && lateral > forward * 0.35;
        if possible_spin {
            let since = self.st.spin_since.get_or_insert_with(Instant::now);
            if !self.st.spin_active && since.elapsed() >= Duration::from_millis(550) {
                let inc = self.st.incident.get_or_insert_with(Incident::new);
                inc.spin = true;
                inc.touch(speed_kmh);
                self.st.spin_active = true;
            }
        } else {
            self.st.spin_since = None;
            self.st.spin_active = false;
        }

        flush_incident(&mut self.st, &mut events);

        let gap_a = valid_gap(self.map.read::<f32>(tbase + T_GAP_AHEAD));
        let gap_b = valid_gap(self.map.read::<f32>(tbase + T_GAP_BEHIND));
        let current_ahead = if place > 1 { rival_at_place(&self.map, place - 1, s_idx, lap_len) } else { None };
        let current_behind = rival_at_place(&self.map, place.saturating_add(1), s_idx, lap_len);

        let close_ahead = gap_a.map(|g| g <= 1.5).unwrap_or(false)
            || current_ahead.as_ref().map(|r| r.distance <= ADJACENT_DISTANCE_M).unwrap_or(false);
        let close_behind = gap_b.map(|g| g <= 1.5).unwrap_or(false)
            || current_behind.as_ref().map(|r| r.distance <= ADJACENT_DISTANCE_M).unwrap_or(false);

        let battle_close = !in_pits && speed_kmh > 40.0 && (close_ahead || close_behind);
        if battle_close {
            let since = self.st.battle_since.get_or_insert_with(Instant::now);
            let cooldown_ok = self.st.last_battle_emit
                .map(|t| t.elapsed() >= BATTLE_COOLDOWN)
                .unwrap_or(true);
            if !self.st.battle_active && cooldown_ok && since.elapsed() >= BATTLE_MIN {
                push_event(&mut events, "CLOSE_BATTLE", true, 60, format!(
                    "ahead={} | behind={} | aheadRival={} | behindRival={}",
                    fmt_gap(gap_a),
                    fmt_gap(gap_b),
                    current_ahead.as_ref().map(|r| r.name.as_str()).unwrap_or("n/a"),
                    current_behind.as_ref().map(|r| r.name.as_str()).unwrap_or("n/a")
                ));
                self.st.battle_active = true;
                self.st.last_battle_emit = Some(Instant::now());
            }
        } else {
            self.st.battle_since = None;
            self.st.battle_active = false;
        }

        self.st.last_ahead = current_ahead;
        self.st.last_behind = current_behind;

        PollResult {
            active: true,
            snapshot: Some(BridgeSnapshot {
                session,
                place,
                in_pits,
                lap_dist,
                speed_kmh,
                gap_ahead_s: gap_a,
                gap_behind_s: gap_b,
            }),
            events,
        }
    }
}

fn push_event(events: &mut Vec<BridgeEvent>, kind: &'static str, eligible: bool, priority: u8, details: String) {
    events.push(BridgeEvent {
        schema_version: BRIDGE_SCHEMA_VERSION,
        kind,
        details,
        highlight_eligible: eligible,
        priority,
    });
}

fn flush_incident(st: &mut State, events: &mut Vec<BridgeEvent>) {
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

        if let Some((kind, priority)) = i.priority() {
            push_event(events, kind, true, priority, details);
        } else {
            push_event(events, "CONTACT_FILTERED", false, 0, format!("{} | no highlight", details));
        }
    }
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
    if place == 0 { return None; }
    let n = map.read::<i32>(OFF_NUM_VEHICLES).clamp(0, MAX_VEHICLES as i32) as usize;
    (0..n).find(|&i| map.read::<u8>(OFF_VEH_SCORING + i * SCORING_STRIDE + SC_PLACE) == place)
}

fn find_vehicle_id(map: &Mapping, id: i32) -> Option<usize> {
    let n = map.read::<i32>(OFF_NUM_VEHICLES).clamp(0, MAX_VEHICLES as i32) as usize;
    (0..n).find(|&i| map.read::<i32>(OFF_VEH_SCORING + i * SCORING_STRIDE + SC_ID) == id)
}

fn race_distance(map: &Mapping, scoring_idx: usize, lap_len: f64) -> f64 {
    let base = OFF_VEH_SCORING + scoring_idx * SCORING_STRIDE;
    let laps = map.read::<i16>(base + SC_TOTAL_LAPS).max(0) as f64;
    let dist = map.read::<f64>(base + SC_LAP_DIST);
    if lap_len.is_finite() && lap_len > 100.0 { laps * lap_len + dist } else { dist }
}

fn rival_at_place(map: &Mapping, place: u8, player_idx: usize, lap_len: f64) -> Option<RivalSnapshot> {
    let idx = find_place(map, place)?;
    if idx == player_idx { return None; }
    let base = OFF_VEH_SCORING + idx * SCORING_STRIDE;
    let id = map.read::<i32>(base + SC_ID);
    let name = map.read_name(base + SC_DRIVER_NAME, 32);
    let in_pits = map.read::<u8>(base + SC_IN_PITS) != 0;
    let player_distance = race_distance(map, player_idx, lap_len);
    let rival_distance = race_distance(map, idx, lap_len);
    Some(RivalSnapshot {
        id,
        name,
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
