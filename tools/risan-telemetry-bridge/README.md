# Risan Telemetry Bridge v0.3

Diagnóstico ultraligero para Le Mans Ultimate. Lee la memoria compartida nativa `LMU_Data` y registra eventos útiles para Risan AutoClips.

## Cambio principal de v0.3

La confirmación de adelantamientos ya no depende del gap de telemetría.

El Bridge guarda la **identidad real (mID) del rival que ocupa la posición inmediatamente delante o detrás**. Si la posición cambia una plaza, espera 2 segundos y comprueba si ese mismo rival intercambió realmente la posición con el jugador.

Ejemplo:

- Tú P5 / rival X P4
- cambio de posición
- tú P4 / el mismo rival X P5 durante 2 s
- resultado: `OVERTAKE_CONFIRMED`

Para una pérdida de posición se aplica el mismo criterio a la inversa.

## Protecciones contra falsos positivos

- Saltos de más de una posición siguen como `POSITION_CHANGE`.
- No confirma si el jugador está en boxes.
- No confirma si el rival estaba o termina en boxes.
- No confirma si el rival desaparece de la sesión.
- La nueva posición debe mantenerse 2 segundos.
- Los gaps negativos siguen descartados.
- Los incidentes continúan agrupados como `INCIDENT_LOW/MEDIUM/HIGH`.
- `CLOSE_BATTLE` sigue teniendo mínimo 5 s y cooldown de 20 s.

## Eventos

- OVERTAKE_CONFIRMED
- POSITION_LOSS_CONFIRMED
- POSITION_CHANGE
- CLOSE_BATTLE
- INCIDENT_LOW / INCIDENT_MEDIUM / INCIDENT_HIGH
- RACE_FINISH

## Uso

1. LMU -> Settings -> Gameplay -> **Enable Plugins** activado.
2. Ejecuta `RisanTelemetryBridge.exe` en el PC donde corre LMU.
3. Haz una tanda normal.
4. Cierra el Bridge al acabar.
5. Pasa `RisanTelemetryEvents.log`.

Trabaja a 10 Hz, no usa GPU y no necesita Internet.

v0.3 sigue siendo diagnóstica. Cuando validemos los adelantamientos, el siguiente paso será enviar únicamente estos eventos por red local a AutoClips.
