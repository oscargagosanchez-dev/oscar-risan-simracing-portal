# Risan LMU Data Bridge v0.1.2 — Contract Test

Herramienta separada de validación. No sustituye Risan Telemetry Bridge v0.5, LMU Hub Live Bridge ni Team Bridge.

Cambios de v0.1.2:
- nuevo evento común `SESSION_CHANGE` cuando cambia el código de sesión de LMU;
- nuevo evento común `SESSION_RESET` cuando LMU reinicia/recarga una sesión aunque el código de sesión permanezca igual;
- detección mediante retroceso claro del reloj de sesión y/o regresión fuerte de vueltas;
- al detectar un límite de sesión se limpia el estado temporal de pits, posiciones, rivales, luchas, incidentes y piloto para evitar falsos eventos heredados;
- se mantienen los arreglos de v0.1.1: FLAG_STATE filtrado, sector flags raw, proximidad circular y log limpio.

Salida:
`RisanLmuDataBridgeContract_v0_1_2.jsonl`

Todo local, sin GPU, Internet ni APIs de pago.
