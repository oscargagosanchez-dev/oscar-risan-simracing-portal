# Risan LMU Data Bridge v0.1 — Contract Test

Herramienta **separada** de validación. No sustituye:

- Risan Telemetry Bridge v0.5 de AutoClips.
- LMU Hub Live Bridge.
- Team Bridge.

Su único objetivo es validar el contrato común antes de plantear una unificación.

## Datos comunes prioritarios

El Core expone y esta herramienta registra:

- driver change
- posición general
- posición por clase (derivada de `mVehicleClass` + clasificación general; debe validarse en multiclase)
- pit entry / pit exit
- incidentes
- banderas globales/sector
- gaps delante/detrás
- vueltas
- timestamp de sesión
- timestamp local de emisión
- freshness
- dedup key / event id

## Eventos comunes

- `DRIVER_CHANGE`
- `PIT_ENTRY`
- `PIT_EXIT`
- `LAP_COMPLETED`
- `POSITION_UPDATE`
- `FLAG_STATE`
- `OVERTAKE_CONFIRMED`
- `POSITION_LOSS_CONFIRMED`
- `POSITION_DUPLICATE_FILTERED`
- `CLOSE_BATTLE`
- `INCIDENT_MEDIUM`
- `INCIDENT_HIGH`
- `CONTACT_FILTERED`
- `RACE_FINISH`

## Freshness

El productor incluye `emitted_unix_ms` y `freshness_ms`. El Contract Test calcula además `receiver_freshness_ms`.

Objetivo inicial local: <= 500 ms.

Cuando haya transporte por red local, el consumidor podrá recalcular freshness sin cambiar el contrato.

## Resistencia por equipos

Antes de sustituir ningún bridge hay que validar en una prueba real:

1. Mismo `car_id`.
2. `DRIVER_CHANGE` de piloto A a piloto B.
3. Posición general correcta.
4. Posición de clase correcta.
5. PIT_ENTRY y PIT_EXIT coherentes.
6. Vuelta/timestamp coherentes.
7. Sin eventos duplicados.
8. Datos dentro del objetivo de freshness.

## Salida

Genera:

`RisanLmuDataBridgeContract_v0_1.jsonl`

Ese archivo se puede pasar al hilo para auditar la prueba.

Todo funciona localmente, sin GPU, Internet ni APIs de pago.
