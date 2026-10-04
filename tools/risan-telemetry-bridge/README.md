# Risan Telemetry Bridge v0.5

Diagnóstico ultraligero para Le Mans Ultimate. Lee la memoria compartida nativa `LMU_Data` y registra eventos útiles para Risan AutoClips.

## Qué cambia en v0.5

La v0.4 ya filtraba correctamente contactos leves. En la última prueba apareció el mismo adelantamiento al mismo rival dos veces con pocos segundos de diferencia, así que v0.5 añade deduplicación específica de eventos de posición.

### Deduplicación de adelantamientos y pérdidas

- Si el mismo rival genera el mismo tipo de evento otra vez dentro de **8 segundos**, el segundo se marca como `POSITION_DUPLICATE_FILTERED`.
- Ese evento duplicado queda solo en el log y **no será elegible para AutoClips**.
- Un evento contrario sí se conserva. Ejemplo: adelantas a un rival y 4 s después él te vuelve a pasar; son dos acciones reales diferentes.
- La confirmación por identidad real del rival y estabilidad de 2 s se mantiene.

## Filtro de contactos

- Impactos inferiores a **80** se ignoran.
- Contacto aislado sin salida/trompo solo es highlight desde **500**.
- Contactos menores quedan como `CONTACT_FILTERED`.
- Salida de pista o trompo conserva el incidente aunque el impacto sea menor.
- Impactos >= 2000, trompo o salida rápida siguen siendo `INCIDENT_HIGH`.

## Eventos elegibles para futuros clips

- OVERTAKE_CONFIRMED
- POSITION_LOSS_CONFIRMED
- CLOSE_BATTLE
- INCIDENT_MEDIUM
- INCIDENT_HIGH
- RACE_FINISH

No elegibles:
- POSITION_CHANGE
- POSITION_DUPLICATE_FILTERED
- CONTACT_FILTERED

## Uso

1. LMU -> Settings -> Gameplay -> **Enable Plugins** activado.
2. Ejecuta `RisanTelemetryBridge.exe` en el PC donde corre LMU.
3. Haz una tanda normal.
4. Cierra el Bridge al acabar.
5. Pasa `RisanTelemetryEvents.log`.

Sigue trabajando a 10 Hz, sin GPU y sin Internet.

Si v0.5 valida la deduplicación, el siguiente paso es conectar estos eventos por red local con Risan AutoClips en el PC de streaming.
