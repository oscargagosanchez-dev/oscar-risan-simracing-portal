#![cfg_attr(not(debug_assertions), windows_subsystem = "console")]

use std::{
    fs::OpenOptions,
    io::Write,
    mem::size_of,
    ptr::read_unaligned,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Memory::{MapViewOfFile, OpenFileMappingW, UnmapViewOfFile, FILE_MAP_READ},
};

const MAP_NAME: &str = "LMU_Data";
const MAX_VEHICLES: usize = 104;

const OFF_SCORING_INFO: usize = 1632;
const OFF_SESSION: usize = OFF_SCORING_INFO + 64;
const OFF_NUM_VEHICLES: usize = OFF_SCORING_INFO + 104;
const OFF_VEH_SCORING: usize = 2192;
const SCORING_STRIDE: usize = 584;

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

const T_ELAPSED: usize = 12;
const T_LOCAL_VEL: usize = 184;
const T_LAST_IMPACT_ET: usize = 552;
const T_LAST_IMPACT_MAG: usize = 560;
const T_GAP_AHEAD: usize = 780;
const T_GAP_BEHIND: usize = 784;
const T_WHEELS: usize = 848;
const WHEEL_STRIDE: usize = 260;
const W_SURFACE_TYPE: usize = 176;

struct Mapping {
    handle: isize,
    ptr: *const u8,
}

impl Mapping {
    fn open() -> Option<Self> {
        let mut wide: Vec<u16> = MAP_NAME.encode_utf16().collect();
        wide.push(0);
        unsafe {
            let handle = OpenFileMappingW(FILE_MAP_READ, 0, wide.as_ptr());
            if handle == 0 {
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

#[derive(Default)]
struct State {
    scoring_idx: Option<usize>,
    last_place: u8,
    last_finish: i8,
    last_impact_et: f64,
    offtrack_since: Option<Instant>,
    spin_since: Option<Instant>,
    battle_since: Option<Instant>,
    battle_active: bool,
}

fn now_string() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    secs.to_string()
}

fn emit(log: &mut std::fs::File, kind: &str, details: &str) {
    let line = format!("{} | {:<20} | {}\n", now_string(), kind, details);
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

fn main() {
    println!("============================================");
    println!("       RISAN TELEMETRY BRIDGE v0.1");
    println!("============================================");
    println!("Diagnostico LMU: no crea clips todavia.");
    println!("CPU objetivo: minimo | GPU: 0 | Internet: 0");
    println!("Log: RisanTelemetryEvents.log\n");

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("RisanTelemetryEvents.log")
        .expect("No se pudo abrir el log");

    emit(&mut log, "BRIDGE_START", "Esperando LMU_Data");

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
            // If LMU closes, the existing mapping can remain valid briefly. We verify basic counts.
            let active = map.read::<u8>(OFF_ACTIVE_VEHICLES) as usize;
            let has_player = map.read::<u8>(OFF_PLAYER_HAS_VEHICLE) != 0;
            let t_idx = map.read::<u8>(OFF_PLAYER_IDX) as usize;

            st.scoring_idx = player_scoring_index(&map, st.scoring_idx);

            if !has_player || t_idx >= MAX_VEHICLES || active == 0 {
                thread::sleep(Duration::from_millis(500));
                // Re-open check catches game shutdown without busy looping.
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
            let place = map.read::<u8>(sbase + SC_PLACE);
            let finish = map.read::<i8>(sbase + SC_FINISH_STATUS);
            let in_pits = map.read::<u8>(sbase + SC_IN_PITS) != 0;
            let lap_dist = map.read::<f64>(sbase + SC_LAP_DIST);
            let path_lat = map.read::<f64>(sbase + SC_PATH_LATERAL);
            let track_edge = map.read::<f64>(sbase + SC_TRACK_EDGE);

            if st.last_place != 0 && place != 0 && place != st.last_place && !in_pits {
                if place < st.last_place {
                    emit(&mut log, "OVERTAKE", &format!("P{} -> P{} | lapDist={:.0}m | session={}", st.last_place, place, lap_dist, session));
                } else {
                    emit(&mut log, "POSITION_LOSS", &format!("P{} -> P{} | lapDist={:.0}m | session={}", st.last_place, place, lap_dist, session));
                }
            }
            st.last_place = place;

            if finish == 1 && st.last_finish != 1 {
                emit(&mut log, "RACE_FINISH", &format!("Final P{} | session={}", place, session));
            }
            st.last_finish = finish;

            let impact_et = map.read::<f64>(tbase + T_LAST_IMPACT_ET);
            let impact_mag = map.read::<f64>(tbase + T_LAST_IMPACT_MAG);
            if impact_et.is_finite() && impact_et > 0.0 && impact_et > st.last_impact_et + 0.001 {
                emit(&mut log, "CONTACT", &format!("magnitude={:.2} | ET={:.2}", impact_mag, impact_et));
                st.last_impact_et = impact_et;
            }

            let vx = map.read::<f64>(tbase + T_LOCAL_VEL);
            let vy = map.read::<f64>(tbase + T_LOCAL_VEL + 8);
            let vz = map.read::<f64>(tbase + T_LOCAL_VEL + 16);
            let speed = (vx*vx + vy*vy + vz*vz).sqrt();
            let speed_kmh = speed * 3.6;

            let mut bad_surface = 0usize;
            for wheel in 0..4 {
                let surface = map.read::<u8>(tbase + T_WHEELS + wheel * WHEEL_STRIDE + W_SURFACE_TYPE);
                if matches!(surface, 2 | 3 | 4) {
                    bad_surface += 1;
                }
            }

            let offtrack = speed_kmh > 20.0 && (bad_surface >= 2 || (track_edge > 0.0 && path_lat.abs() > track_edge + 0.5));
            if offtrack {
                let since = st.offtrack_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= Duration::from_millis(450) {
                    emit(&mut log, "OFF_TRACK", &format!("{} ruedas fuera | {:.0} km/h", bad_surface, speed_kmh));
                    st.offtrack_since = None;
                    thread::sleep(Duration::from_millis(900));
                }
            } else {
                st.offtrack_since = None;
            }

            let forward = vz.abs();
            let lateral = vx.abs();
            let possible_spin = speed_kmh > 45.0 && lateral > 6.0 && lateral > forward * 0.35;
            if possible_spin {
                let since = st.spin_since.get_or_insert_with(Instant::now);
                if since.elapsed() >= Duration::from_millis(550) {
                    emit(&mut log, "POSSIBLE_SPIN", &format!("lateral={:.1} m/s | {:.0} km/h", lateral, speed_kmh));
                    st.spin_since = None;
                    thread::sleep(Duration::from_millis(1200));
                }
            } else {
                st.spin_since = None;
            }

            let gap_a = map.read::<f32>(tbase + T_GAP_AHEAD);
            let gap_b = map.read::<f32>(tbase + T_GAP_BEHIND);
            let close = !in_pits && speed_kmh > 40.0 &&
                ((gap_a.is_finite() && gap_a > 0.0 && gap_a <= 1.0) ||
                 (gap_b.is_finite() && gap_b > 0.0 && gap_b <= 1.0));

            if close {
                let since = st.battle_since.get_or_insert_with(Instant::now);
                if !st.battle_active && since.elapsed() >= Duration::from_secs(5) {
                    emit(&mut log, "CLOSE_BATTLE", &format!("ahead={:.2}s | behind={:.2}s", gap_a, gap_b));
                    st.battle_active = true;
                }
            } else {
                st.battle_since = None;
                st.battle_active = false;
            }

            // 10 Hz is more than enough for highlight events and keeps load tiny.
            thread::sleep(Duration::from_millis(100));
        }

        thread::sleep(Duration::from_secs(1));
    }
}
