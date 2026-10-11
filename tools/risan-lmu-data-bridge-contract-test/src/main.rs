use std::{
    fs::OpenOptions,
    io::Write,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use risan_lmu_data_bridge_core::{
    LmuDataBridgeCore, CONTRACT_VERSION, CORE_VERSION, FRESHNESS_TARGET_MS, SOURCE_NAME,
};
use serde_json::json;

fn unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn write_json_line(file: &mut std::fs::File, value: serde_json::Value) {
    if let Ok(line) = serde_json::to_string(&value) {
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}

fn main() {
    println!("====================================================");
    println!(" RISAN LMU DATA BRIDGE v0.1 - CONTRACT TEST");
    println!("====================================================");
    println!("NO sustituye AutoClips Bridge ni LMU Hub Live Bridge.");
    println!("Contrato: {} | Core: {}", CONTRACT_VERSION, CORE_VERSION);
    println!("Fuente: {}", SOURCE_NAME);
    println!("Freshness objetivo: <= {} ms", FRESHNESS_TARGET_MS);
    println!("Local, sin GPU, sin Internet, sin creditos.");
    println!("Log JSONL: RisanLmuDataBridgeContract_v0_1.jsonl\n");

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("RisanLmuDataBridgeContract_v0_1.jsonl")
        .expect("No se pudo abrir el log");

    write_json_line(&mut log, json!({
        "record_type": "contract_test_start",
        "contract_version": CONTRACT_VERSION,
        "core_version": CORE_VERSION,
        "source": SOURCE_NAME,
        "emitted_unix_ms": unix_ms(),
        "freshness_target_ms": FRESHNESS_TARGET_MS
    }));

    loop {
        let Some(mut core) = LmuDataBridgeCore::connect() else {
            print!("\rEsperando LMU_Data...                     ");
            let _ = std::io::stdout().flush();
            thread::sleep(Duration::from_secs(2));
            continue;
        };

        println!("\nLMU_Data detectado. Validacion del contrato activa.");
        let mut last_snapshot_write = Instant::now() - Duration::from_secs(2);

        loop {
            let result = core.poll();

            for event in result.events {
                let received = unix_ms();
                let receiver_freshness_ms = received.saturating_sub(event.emitted_unix_ms);
                let fresh = receiver_freshness_ms <= FRESHNESS_TARGET_MS as u128;

                println!(
                    "{} | fresh={}ms | P{:?}/C{:?} | lap={:?} | driver={:?}",
                    event.event_type,
                    receiver_freshness_ms,
                    event.overall_position,
                    event.class_position,
                    event.lap,
                    event.driver_name
                );

                write_json_line(&mut log, json!({
                    "record_type": "event",
                    "receiver_received_unix_ms": received,
                    "receiver_freshness_ms": receiver_freshness_ms,
                    "fresh": fresh,
                    "event": event
                }));
            }

            if let Some(snapshot) = result.snapshot {
                if last_snapshot_write.elapsed() >= Duration::from_secs(1) {
                    let received = unix_ms();
                    let receiver_freshness_ms = received.saturating_sub(snapshot.emitted_unix_ms);
                    write_json_line(&mut log, json!({
                        "record_type": "snapshot",
                        "receiver_received_unix_ms": received,
                        "receiver_freshness_ms": receiver_freshness_ms,
                        "fresh": receiver_freshness_ms <= FRESHNESS_TARGET_MS as u128,
                        "snapshot": snapshot
                    }));
                    last_snapshot_write = Instant::now();
                }
            }

            if !result.active {
                thread::sleep(Duration::from_millis(500));
                if !LmuDataBridgeCore::source_available() {
                    write_json_line(&mut log, json!({
                        "record_type": "source_disconnected",
                        "emitted_unix_ms": unix_ms()
                    }));
                    println!("LMU_Data desconectado. Esperando siguiente sesion.");
                    break;
                }
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        }
    }
}
