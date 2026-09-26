# Risan Telemetry Bridge v0.1

Diagnóstico ultraligero para Le Mans Ultimate. Lee la memoria compartida nativa `LMU_Data` y registra eventos útiles para Risan AutoClips.

## Objetivo de esta versión

No crea clips todavía. Solo valida que LMU entrega correctamente:

- OVERTAKE
- POSITION_LOSS
- CONTACT
- OFF_TRACK
- POSSIBLE_SPIN
- CLOSE_BATTLE
- RACE_FINISH

Los eventos se ven en la ventana y se guardan en `RisanTelemetryEvents.log`.

## Requisitos LMU

1. LMU -> Settings -> Gameplay.
2. Activa **Enable Plugins**.
3. Reinicia LMU completamente.

Las versiones actuales de LMU publican su propia memoria compartida `LMU_Data`; no debería hacer falta instalar ningún DLL adicional.

## Uso

1. Ejecuta `RisanTelemetryBridge.exe` en el PC donde corre LMU.
2. Puedes abrirlo antes o después de LMU.
3. Déjalo abierto durante la carrera.
4. Al terminar, cierra con la X o Ctrl+C.
5. Conserva `RisanTelemetryEvents.log`.

El bridge trabaja a 10 Hz, no usa GPU y no necesita Internet.

## Nota

v0.1 es deliberadamente diagnóstica. Primero verificamos detección y consumo. Después conectaremos estos eventos a AutoClips en el PC de streaming.
