# Risan Telemetry Bridge v0.6

Esta versión deja de mantener su propia lógica LMU aislada.

El lector y la interpretación común pasan a **Risan LMU Data Bridge Core**, siguiendo la especificación compartida definida para LMU Hub.

## Arquitectura

`LMU_Data -> Risan LMU Data Bridge Core -> consumidores`

Consumidores previstos:
- LMU Hub
- Team Radio
- AutoClips
- Spotter
- Broadcast
- Strategy / Race Events

Risan Telemetry Bridge es ahora un adaptador de diagnóstico para AutoClips: consume el Core común y escribe los eventos en `RisanTelemetryEvents.log`.

## Contrato compartido

Cada evento expone:
- schema_version
- kind
- details
- highlight_eligible
- priority

Además el Core expone un snapshot común con:
- session
- place
- in_pits
- lap_dist
- speed_kmh
- gap_ahead_s
- gap_behind_s

## Lógica centralizada

Se ha movido al Core común:
- lectura de `LMU_Data`
- detección de jugador
- identificación de rivales
- confirmación de adelantamientos/pérdidas
- exclusión de boxes
- deduplicación
- gaps válidos
- CLOSE_BATTLE
- contacto / off-track / spin
- agrupación y prioridad de incidentes

AutoClips ya no debe duplicar estas reglas.

## Recursos

- 10 Hz
- GPU: 0
- Internet: 0
- procesamiento local

## Próximo paso

La conexión por red local con AutoClips deberá transportar estos mismos eventos del Core, sin volver a implementar detección de carrera dentro de AutoClips.
