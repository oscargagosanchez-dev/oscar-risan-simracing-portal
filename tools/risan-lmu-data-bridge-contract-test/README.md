# Risan LMU Data Bridge v0.1.1 — Contract Test

Herramienta separada de validación. No sustituye Risan Telemetry Bridge v0.5, LMU Hub Live Bridge ni Team Bridge.

Cambios:
- log limpio en cada ejecución;
- FLAG_STATE solo por cambios globales (game_phase / yellow_flag_state);
- sector_flags se conserva como dato raw de snapshot, sin emitir eventos por sí solo;
- proximidad en pista calculada de forma circular sobre la vuelta;
- se mantienen freshness, timestamps, posiciones general/clase, pits, vueltas, incidentes, deduplicación y driver change.

Salida: RisanLmuDataBridgeContract_v0_1_1.jsonl

Todo local, sin GPU, Internet ni APIs de pago.
