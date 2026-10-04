# Risan Telemetry Bridge v0.4

Diagnóstico ultraligero para Le Mans Ultimate. Lee la memoria compartida nativa `LMU_Data` y registra eventos útiles para Risan AutoClips.

## Qué cambia en v0.4

La v0.3 confirmó correctamente adelantamientos, pérdidas de posición e ignoró los cambios producidos en boxes. La v0.4 afina los incidentes para evitar clips por roces o golpes irrelevantes.

### Filtro de contactos

- Impactos inferiores a **80** se ignoran por completo como ruido/roce mínimo.
- Un contacto aislado sin salida de pista ni trompo solo se considera highlight desde **500** de magnitud.
- Los contactos menores de 500 quedan como `CONTACT_FILTERED` únicamente para diagnóstico y **no deben disparar AutoClips**.
- Si existe salida de pista o trompo, el incidente sí se conserva aunque el impacto sea pequeño.
- Impacto >= 2000, trompo o salida a alta velocidad siguen siendo `INCIDENT_HIGH`.

## Se mantiene

- `OVERTAKE_CONFIRMED` y `POSITION_LOSS_CONFIRMED` por intercambio de identidad real del rival.
- Cambios en boxes excluidos de highlights.
- Saltos múltiples de posición como `POSITION_CHANGE`.
- Incidentes agrupados.
- `CLOSE_BATTLE` con mínimo de 5 s y cooldown de 20 s.
- 10 Hz, GPU 0, sin Internet.

## Eventos elegibles para futuros clips

- OVERTAKE_CONFIRMED
- POSITION_LOSS_CONFIRMED
- CLOSE_BATTLE
- INCIDENT_MEDIUM
- INCIDENT_HIGH
- RACE_FINISH

`POSITION_CHANGE` y `CONTACT_FILTERED` son diagnóstico y no serán triggers de clip.

## Uso

1. LMU -> Settings -> Gameplay -> **Enable Plugins** activado.
2. Ejecuta `RisanTelemetryBridge.exe` en el PC donde corre LMU.
3. Haz una tanda normal.
4. Cierra el Bridge al acabar.
5. Pasa `RisanTelemetryEvents.log`.

v0.4 sigue siendo diagnóstica. Si el filtrado queda bien, el siguiente paso es enviar solo los eventos elegibles por red local al AutoClips del PC de streaming.
