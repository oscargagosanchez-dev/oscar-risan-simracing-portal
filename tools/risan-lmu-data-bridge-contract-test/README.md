# Risan LMU Data Bridge v0.1.3 — Contract Test

Herramienta separada de validación. No sustituye Risan Telemetry Bridge v0.5, LMU Hub Live Bridge ni Team Bridge.

Cambios de v0.1.3:
- cancela cualquier adelantamiento/pérdida pendiente cuando el jugador entra en boxes;
- añade `POSITION_PENDING_CANCELLED` como evento diagnóstico no elegible para highlight;
- bloquea highlights de posición durante 5 segundos después de `SESSION_CHANGE` o `SESSION_RESET`;
- durante ese guard, los cambios de posición siguen registrándose como `POSITION_CHANGE`, pero nunca como highlight;
- mantiene SESSION_CHANGE / SESSION_RESET, freshness, pits, vueltas, posiciones general/clase, incidentes, deduplicación y driver change.

Salida:
`RisanLmuDataBridgeContract_v0_1_3.jsonl`

Todo local, sin GPU, Internet ni APIs de pago.
