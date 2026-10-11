use std::{
    ptr::read_unaligned,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::Memory::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ},
};

pub const CONTRACT_VERSION: &str = "0.1";
pub const CORE_VERSION: &str = "0.1.1";
pub const SOURCE_NAME: &str = "LMU_Data";
pub const SOURCE_BRIDGE: &str = "risan-lmu-data-bridge-core";
pub const FRESHNESS_TARGET_MS: u64 = 500;

const MAX_VEHICLES: usize = 104;

const OFF_SCORING_INFO: usize = 1632;
const OFF_SESSION: usize = OFF_SCORING_INFO + 64;
const OFF_CURRENT_ET: usize = OFF_SCORING_INFO + 68;
const OFF_LAP_LENGTH: usize = OFF_SCORING_INFO + 88;
const OFF_NUM_VEHICLES: usize = OFF_SCORING_INFO + 104;
const OFF_GAME_PHASE: usize = OFF_SCORING_INFO + 108;
const OFF_YELLOW_FLAG_STATE: usize = OFF_SCORING_INFO + 109;
const OFF_SECTOR_FLAGS: usize = OFF_SCORING_INFO + 110;

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
const SC_VEHICLE_CLASS: usize = 200;

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

static EVENT_SEQ: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Debug, Serialize)]
pub struct BridgeEvent {
    pub contract_version: &'static str,
    pub source_bridge: &'static str,
    pub event_id: String,
    pub event_type: &'static str,
    pub session_timestamp_s: f64,
    pub emitted_unix_ms: u128,
    pub freshness_ms: u64,
    pub dedup_key: String,

    pub car_id: Option<i32>,
    pub driver_name: Option<String>,
    pub previous_driver_name: Option<String>,
    pub vehicle_class: Option<String>,

    pub overall_position: Option<u8>,
    pub class_position: Option<u8>,
    pub lap: Option<i16>,
    pub in_pits: Option<bool>,

    pub priority: u8,
    pub confidence: u8,
    pub highlight_eligible: bool,
    pub details: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct BridgeSnapshot {
    pub contract_version: &'static str,
    pub source_bridge: &'static str,
    pub source_name: &'static str,

    pub session: i32,
    pub session_timestamp_s: f64,
    pub emitted_unix_ms: u128,
    pub freshness_ms: u64,

    pub car_id: i32,
    pub driver_name: String,
    pub vehicle_class: String,
    pub overall_position: u8,
    pub class_position: Option<u8>,

    pub lap: i16,
    pub in_pits: bool,
    pub lap_dist: f64,
    pub speed_kmh: f64,
    pub gap_ahead_s: Option<f32>,
    pub gap_behind_s: Option<f32>,

    pub game_phase: u8,
    pub yellow_flag_state: i8,
    pub sector_flags: [i8; 3],
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
    last_class_position: Option<u8>,
    last_finish: i8,
    last_lap: Option<i16>,
    last_in_pits: Option<bool>,
    last_driver_name: Option<String>,
    last_car_id: Option<i32>,
    last_flags: Option<(u8, i8, [i8; 3])>,

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
            flush_incident(&mut self.st, &mut events, 0.0, None);
            return PollResult { active: false, snapshot: None, events };
        }

        let Some(s_idx) = self.st.scoring_idx else {
            return PollResult { active: false, snapshot: None, events };
        };

        let sbase = OFF_VEH_SCORING + s_idx * SCORING_STRIDE;
        let tbase = OFF_TELEM_INFO + t_idx * TELEMETRY_STRIDE;

        let session = self.map.read::<i32>(OFF_SESSION);
        let session_ts = self.map.read::<f64>(OFF_CURRENT_ET);
        let lap_len = self.map.read::<f64>(OFF_LAP_LENGTH);
        let game_phase = self.map.read::<u8>(OFF_GAME_PHASE);
        let yellow_flag_state = self.map.read::<i8>(OFF_YELLOW_FLAG_STATE);
        let sector_flags = [
            self.map.read::<i8>(OFF_SECTOR_FLAGS),
            self.map.read::<i8>(OFF_SECTOR_FLAGS + 1),
            self.map.read::<i8>(OFF_SECTOR_FLAGS + 2),
        ];

        let car_id = self.map.read::<i32>(sbase + SC_ID);
        let driver_name = self.map.read_name(sbase + SC_DRIVER_NAME, 32);
        let vehicle_class = self.map.read_name(sbase + SC_VEHICLE_CLASS, 32);
        let place = self.map.read::<u8>(sbase + SC_PLACE);
        let class_position = class_position(&self.map, s_idx, &vehicle_class);
        let lap = self.map.read::<i16>(sbase + SC_TOTAL_LAPS);
        let finish = self.map.read::<i8>(sbase + SC_FINISH_STATUS);
        let in_pits = self.map.read::<u8>(sbase + SC_IN_PITS) != 0;
        let lap_dist = self.map.read::<f64>(sbase + SC_LAP_DIST);
        let path_lat = self.map.read::<f64>(sbase + SC_PATH_LATERAL);
        let track_edge = self.map.read::<f64>(sbase + SC_TRACK_EDGE);

        let vx = self.map.read::<f64>(tbase + T_LOCAL_VEL);
        let vy = self.map.read::<f64>(tbase + T_LOCAL_VEL + 8);
        let vz = self.map.read::<f64>(tbase + T_LOCAL_VEL + 16);
        let speed_kmh = (vx * vx + vy * vy + vz * vz).sqrt() * 3.6;

        let base_ctx = EventContext {
            session_timestamp_s: session_ts,
            car_id: Some(car_id),
            driver_name: nonempty(&driver_name),
            previous_driver_name: None,
            vehicle_class: nonempty(&vehicle_class),
            overall_position: if place > 0 { Some(place) } else { None },
            class_position,
            lap: Some(lap),
            in_pits: Some(in_pits),
        };

        // Common state event: driver change, intentionally independent of AutoClips highlight logic.
        if let (Some(prev_car), Some(prev_driver)) = (self.st.last_car_id, self.st.last_driver_name.as_ref()) {
            if prev_car == car_id && !driver_name.is_empty() && prev_driver != &driver_name {
                let mut ctx = base_ctx.clone();
                ctx.previous_driver_name = Some(prev_driver.clone());
                push_event(
                    &mut events,
                    "DRIVER_CHANGE",
                    false,
                    95,
                    95,
                    format!("driver_change:{}:{}", car_id, driver_name),
                    format!("{} -> {} | car_id={}", prev_driver, driver_name, car_id),
                    ctx,
                );
            }
        }
        if !driver_name.is_empty() {
            self.st.last_driver_name = Some(driver_name.clone());
        }
        self.st.last_car_id = Some(car_id);

        // Common pit lifecycle.
        if let Some(prev) = self.st.last_in_pits {
            if !prev && in_pits {
                push_event(
                    &mut events,
                    "PIT_ENTRY",
                    false,
                    80,
                    100,
                    format!("pit:{}:{}:entry", car_id, lap),
                    format!("pit entry | P{} | lap={}", place, lap),
                    base_ctx.clone(),
                );
            } else if prev && !in_pits {
                push_event(
                    &mut events,
                    "PIT_EXIT",
                    false,
                    80,
                    100,
                    format!("pit:{}:{}:exit", car_id, lap),
                    format!("pit exit | P{} | lap={}", place, lap),
                    base_ctx.clone(),
                );
            }
        }
        self.st.last_in_pits = Some(in_pits);

        // Common lap lifecycle.
        if let Some(prev_lap) = self.st.last_lap {
            if lap > prev_lap {
                push_event(
                    &mut events,
                    "LAP_COMPLETED",
                    false,
                    45,
                    100,
                    format!("lap:{}:{}", car_id, lap),
                    format!("lap {} completed | P{} | class={:?}", lap, place, class_position),
                    base_ctx.clone(),
                );
            }
        }
        self.st.last_lap = Some(lap);

        // General/class position state. This is deliberately separate from confirmed overtake events.
        if self.st.last_place != 0 && (place != self.st.last_place || class_position != self.st.last_class_position) {
            push_event(
                &mut events,
                "POSITION_UPDATE",
                false,
                55,
                95,
                format!("position:{}:{}:{:?}:{}", car_id, place, class_position, lap),
                format!(
                    "overall P{} -> P{} | class {:?} -> {:?}",
                    self.st.last_place, place, self.st.last_class_position, class_position
                ),
                base_ctx.clone(),
            );
        }
        self.st.last_class_position = class_position;

        // Common flags state.
        let flag_state = (game_phase, yellow_flag_state, sector_flags);
        if self.st.last_flags.map(|f| f != flag_state).unwrap_or(true) {
            push_event(
                &mut events,
                "FLAG_STATE",
                false,
                70,
                100,
                format!(
                    "flags:{}:{}:{}:{}:{}",
                    session, game_phase, yellow_flag_state, sector_flags[0], sector_flags[1]
                ),
                format!(
                    "game_phase={} | yellow={} | sectors={:?}",
                    game_phase, yellow_flag_state, sector_flags
                ),
                base_ctx.clone(),
            );
            self.st.last_flags = Some(flag_state);
        }

        // Existing, validated AutoClips-compatible position confirmation logic.
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
                        push_event(
                            &mut events,
                            "POSITION_CHANGE",
                            false,
                            0,
                            100,
                            format!("position-change:{}:{}:{}", car_id, self.st.last_place, place),
                            format!("P{} -> P{} | rival={} in pits | no highlight", self.st.last_place, place, rival.name),
                            base_ctx.clone(),
                        );
                        self.st.pending_position = None;
                    }
                } else {
                    push_event(
                        &mut events,
                        "POSITION_CHANGE",
                        false,
                        0,
                        80,
                        format!("position-change:{}:{}:{}", car_id, self.st.last_place, place),
                        format!("P{} -> P{} | rival identity unavailable | no highlight", self.st.last_place, place),
                        base_ctx.clone(),
                    );
                    self.st.pending_position = None;
                }
            } else {
                push_event(
                    &mut events,
                    "POSITION_CHANGE",
                    false,
                    0,
                    100,
                    format!("position-change:{}:{}:{}", car_id, self.st.last_place, place),
                    format!("P{} -> P{} | delta={} | pits={} | no highlight", self.st.last_place, place, delta, in_pits),
                    base_ctx.clone(),
                );
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
                            push_event(
                                &mut events,
                                "POSITION_DUPLICATE_FILTERED",
                                false,
                                0,
                                100,
                                format!("position-dedup:{}:{}:{}", car_id, p.rival.id, overtake),
                                format!("P{} -> P{} | rival={} | duplicate filtered", p.old, p.new, p.rival.name),
                                base_ctx.clone(),
                            );
                        } else {
                            let kind = if overtake { "OVERTAKE_CONFIRMED" } else { "POSITION_LOSS_CONFIRMED" };
                            push_event(
                                &mut events,
                                kind,
                                true,
                                85,
                                98,
                                format!("pass:{}:{}:{}:{}", car_id, p.rival.id, overtake, lap),
                                format!(
                                    "P{} -> P{} | rival={} (id={}) swapped to P{} | stable=2s | preDistance={:.0}m | lapDist={:.0}m | session={}",
                                    p.old, p.new, p.rival.name, p.rival.id, rival_place, p.rival.distance, p.lap_dist, p.session
                                ),
                                base_ctx.clone(),
                            );
                            self.st.last_position_event = Some(LastPositionEvent {
                                rival_id: p.rival.id,
                                overtake,
                                at: Instant::now(),
                            });
                        }
                        self.st.pending_position = None;
                    }
                    Some((rival_place, rival_in_pits)) => {
                        push_event(
                            &mut events,
                            "POSITION_CHANGE",
                            false,
                            0,
                            95,
                            format!("position-change:{}:{}:{}", car_id, p.old, p.new),
                            format!("P{} -> P{} | rival={} now P{} pits={} | swap not confirmed", p.old, p.new, p.rival.name, rival_place, rival_in_pits),
                            base_ctx.clone(),
                        );
                        self.st.pending_position = None;
                    }
                    None => {
                        push_event(
                            &mut events,
                            "POSITION_CHANGE",
                            false,
                            0,
                            80,
                            format!("position-change:{}:{}:{}", car_id, p.old, p.new),
                            format!("P{} -> P{} | rival={} disappeared | no highlight", p.old, p.new, p.rival.name),
                            base_ctx.clone(),
                        );
                        self.st.pending_position = None;
                    }
                }
            }
        }

        if finish == 1 && self.st.last_finish != 1 {
            push_event(
                &mut events,
                "RACE_FINISH",
                true,
                90,
                100,
                format!("finish:{}:{}", car_id, lap),
                format!("Final P{} | session={}", place, session),
                base_ctx.clone(),
            );
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

        flush_incident(&mut self.st, &mut events, session_ts, Some(base_ctx.clone()));

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
                push_event(
                    &mut events,
                    "CLOSE_BATTLE",
                    true,
                    60,
                    90,
                    format!("battle:{}:{}:{}", car_id, lap, (session_ts * 10.0) as i64),
                    format!(
                        "ahead={} | behind={} | aheadRival={} | behindRival={}",
                        fmt_gap(gap_a),
                        fmt_gap(gap_b),
                        current_ahead.as_ref().map(|r| r.name.as_str()).unwrap_or("n/a"),
                        current_behind.as_ref().map(|r| r.name.as_str()).unwrap_or("n/a")
                    ),
                    base_ctx.clone(),
                );
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
                contract_version: CONTRACT_VERSION,
                source_bridge: SOURCE_BRIDGE,
                source_name: SOURCE_NAME,
                session,
                session_timestamp_s: session_ts,
                emitted_unix_ms: unix_ms(),
                freshness_ms: 0,
                car_id,
                driver_name,
                vehicle_class,
                overall_position: place,
                class_position,
                lap,
                in_pits,
                lap_dist,
                speed_kmh,
                gap_ahead_s: gap_a,
                gap_behind_s: gap_b,
                game_phase,
                yellow_flag_state,
                sector_flags,
            }),
            events,
        }
    }
}

#[derive(Clone)]
struct EventContext {
    session_timestamp_s: f64,
    car_id: Option<i32>,
    driver_name: Option<String>,
    previous_driver_name: Option<String>,
    vehicle_class: Option<String>,
    overall_position: Option<u8>,
    class_position: Option<u8>,
    lap: Option<i16>,
    in_pits: Option<bool>,
}

fn push_event(
    events: &mut Vec<BridgeEvent>,
    event_type: &'static str,
    highlight_eligible: bool,
    priority: u8,
    confidence: u8,
    dedup_key: String,
    details: String,
    ctx: EventContext,
) {
    let emitted = unix_ms();
    let seq = EVENT_SEQ.fetch_add(1, Ordering::Relaxed);
    events.push(BridgeEvent {
        contract_version: CONTRACT_VERSION,
        source_bridge: SOURCE_BRIDGE,
        event_id: format!("{}-{}-{}", event_type, emitted, seq),
        event_type,
        session_timestamp_s: ctx.session_timestamp_s,
        emitted_unix_ms: emitted,
        freshness_ms: 0,
        dedup_key,
        car_id: ctx.car_id,
        driver_name: ctx.driver_name,
        previous_driver_name: ctx.previous_driver_name,
        vehicle_class: ctx.vehicle_class,
        overall_position: ctx.overall_position,
        class_position: ctx.class_position,
        lap: ctx.lap,
        in_pits: ctx.in_pits,
        priority,
        confidence,
        highlight_eligible,
        details,
    });
}

fn flush_incident(
    st: &mut State,
    events: &mut Vec<BridgeEvent>,
    session_ts: f64,
    ctx: Option<EventContext>,
) {
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

        let mut ctx = ctx.unwrap_or(EventContext {
            session_timestamp_s: session_ts,
            car_id: None,
            driver_name: None,
            previous_driver_name: None,
            vehicle_class: None,
            overall_position: None,
            class_position: None,
            lap: None,
            in_pits: None,
        });
        ctx.session_timestamp_s = session_ts;

        if let Some((kind, priority)) = i.priority() {
            push_event(
                events,
                kind,
                true,
                priority,
                95,
                format!(
                    "incident:{}:{}:{}",
                    ctx.car_id.unwrap_or(-1),
                    (session_ts * 10.0) as i64,
                    priority
                ),
                details,
                ctx,
            );
        } else {
            push_event(
                events,
                "CONTACT_FILTERED",
                false,
                0,
                100,
                format!("contact-filtered:{}:{}", ctx.car_id.unwrap_or(-1), (session_ts * 10.0) as i64),
                format!("{} | no highlight", details),
                ctx,
            );
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

fn class_position(map: &Mapping, player_idx: usize, class_name: &str) -> Option<u8> {
    if class_name.is_empty() { return None; }
    let base = OFF_VEH_SCORING + player_idx * SCORING_STRIDE;
    let player_place = map.read::<u8>(base + SC_PLACE);
    if player_place == 0 { return None; }

    let n = map.read::<i32>(OFF_NUM_VEHICLES).clamp(0, MAX_VEHICLES as i32) as usize;
    let mut ahead_same_class: u8 = 0;
    for i in 0..n {
        if i == player_idx { continue; }
        let b = OFF_VEH_SCORING + i * SCORING_STRIDE;
        let p = map.read::<u8>(b + SC_PLACE);
        if p == 0 || p >= player_place { continue; }
        let c = map.read_name(b + SC_VEHICLE_CLASS, 32);
        if c == class_name {
            ahead_same_class = ahead_same_class.saturating_add(1);
        }
    }
    Some(ahead_same_class.saturating_add(1))
}

fn valid_gap(g: f32) -> Option<f32> {
    if g.is_finite() && g > 0.0 && g <= 20.0 { Some(g) } else { None }
}

fn fmt_gap(g: Option<f32>) -> String {
    g.map(|v| format!("{v:.2}s")).unwrap_or_else(|| "n/a".to_string())
}

fn nonempty(s: &str) -> Option<String> {
    if s.is_empty() { None } else { Some(s.to_string()) }
}

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
