use std::{
    fs::OpenOptions,
    io::Write,
    thread,
    time::Duration,
};

use risan_lmu_data_bridge_core::{
    LmuDataBridgeCore, BRIDGE_SCHEMA_VERSION, SOURCE_NAME,
};

use windows_sys::Win32::{
    Foundation::SYSTEMTIME,
    System::SystemInformation::GetLocalTime,
};

fn now_string() -> String {
    unsafe {
        let mut t: SYSTEMTIME = std::mem::zeroed();
        GetLocalTime(&mut t);
        format!("{:02}:{:02}:{:02}", t.wHour, t.wMinute, t.wSecond)
    }
}

fn log_line(log: &mut std::fs::File, kind: &str, details: &str) {
    let line = format!("{} | {:<28} | {}\n", now_string(), kind, details);
    print!("{line}");
    let _ = log.write_all(line.as_bytes());
    let _ = log.flush();
}

fn main() {
    println!("============================================");
    println!("       RISAN TELEMETRY BRIDGE v0.6");
    println!("============================================");
    println!("Basado en Risan LMU Data Bridge Core.");
    println!("Schema compartido: {}", BRIDGE_SCHEMA_VERSION);
    println!("Fuente: {}", SOURCE_NAME);
    println!("CPU objetivo: minimo | GPU: 0 | Internet: 0");
    println!("Log: RisanTelemetryEvents.log\n");

    let mut log = OpenOptions::new()
        .create(true)
        .append(true)
        .open("RisanTelemetryEvents.log")
        .expect("No se pudo abrir el log");

    log_line(&mut log, "BRIDGE_START", &format!(
        "v0.6 | core-schema={} | Esperando {}",
        BRIDGE_SCHEMA_VERSION, SOURCE_NAME
    ));

    loop {
        let Some(mut core) = LmuDataBridgeCore::connect() else {
            print!("\rEsperando a Le Mans Ultimate...          ");
            let _ = std::io::stdout().flush();
            thread::sleep(Duration::from_secs(2));
            continue;
        };

        println!("\nLMU_Data detectado. Core compartido activo.");
        log_line(&mut log, "LMU_CONNECTED", "Risan LMU Data Bridge Core conectado");

        loop {
            let result = core.poll();

            for event in result.events {
                log_line(
                    &mut log,
                    event.kind,
                    &format!(
                        "{} | eligible={} | priority={} | schema={}",
                        event.details,
                        event.highlight_eligible,
                        event.priority,
                        event.schema_version
                    ),
                );
            }

            if !result.active {
                thread::sleep(Duration::from_millis(500));
                if !LmuDataBridgeCore::source_available() {
                    log_line(&mut log, "LMU_DISCONNECTED", "Esperando siguiente sesion");
                    break;
                }
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        }

        thread::sleep(Duration::from_secs(1));
    }
}
