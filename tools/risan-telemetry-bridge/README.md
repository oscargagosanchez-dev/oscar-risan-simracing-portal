# Risan Telemetry Bridge v0.2

Diagnóstico ultraligero para Le Mans Ultimate. Lee la memoria compartida nativa `LMU_Data` y registra eventos útiles para Risan AutoClips.

## Qué mejora respecto a v0.1

- Hora local legible (`HH:MM:SS`).
- Ignora gaps negativos/no válidos.
- Un cambio de posición ya no se considera automáticamente adelantamiento.
- `OVERTAKE_CONFIRMED` / `POSITION_LOSS_CONFIRMED` requieren:
  - cambio de una sola posición,
  - rival cercano antes del cambio,
  - jugador fuera de boxes,
  - mantener la nueva posición durante 2 segundos.
- Cambios que no cumplen las condiciones quedan como `POSITION_CHANGE` y no serán highlight.
- Contactos + salida + trompo se agrupan en un único `INCIDENT_LOW/MEDIUM/HIGH`.
- `CLOSE_BATTLE` requiere 5 s de proximidad y tiene 20 s de cooldown.
- Sin sleeps largos dentro de la detección: el muestreo sigue estable a 10 Hz.

## Eventos

- OVERTAKE_CONFIRMED
- POSITION_LOSS_CONFIRMED
- POSITION_CHANGE
- CLOSE_BATTLE
- INCIDENT_LOW / INCIDENT_MEDIUM / INCIDENT_HIGH
- RACE_FINISH

## Requisitos LMU

1. LMU -> Settings -> Gameplay.
2. Activa **Enable Plugins**.
3. Reinicia LMU completamente si acabas de cambiarlo.

## Uso

1. Ejecuta `RisanTelemetryBridge.exe` en el PC donde corre LMU.
2. Puedes abrirlo antes o después de LMU.
3. Déjalo abierto durante una tanda normal.
4. Al terminar, cierra con la X o Ctrl+C.
5. Pasa `RisanTelemetryEvents.log` para revisar la prueba.

El bridge trabaja a 10 Hz, no usa GPU y no necesita Internet.

## Importante

v0.2 sigue siendo diagnóstica: todavía no crea clips. Primero validamos fiabilidad y consumo; después conectaremos estos eventos con AutoClips en el PC de streaming por red local.
