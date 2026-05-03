# Validación de Modo ArduPilot en SITL

## 1. Objetivo

Definir cómo se observará y validará el `custom_mode` de ArduPilot SITL para que la regla `ARM-AUTO-001` pueda distinguir entre modo automático, modo no automático y estado desconocido.

Este documento aplica a MVP 0.1 y solo a ArduPilot Copter SITL. No valida PX4, Plane, Rover ni hardware real.

## 2. Base Técnica

ArduPilot documenta que el modo de vuelo actual se envía una vez por segundo en el campo `custom_mode` del mensaje MAVLink `HEARTBEAT`. El número de modo varía por tipo de vehículo, por lo que debe interpretarse según la tabla de modos del vehículo concreto.

Para ArduPilot Copter, la documentación de parámetros `FLTMODE*` lista los valores de modo relevantes:

| `custom_mode` | Modo Copter |
|---:|---|
| 0 | Stabilize |
| 1 | Acro |
| 2 | AltHold |
| 3 | Auto |
| 4 | Guided |
| 5 | Loiter |
| 6 | RTL |
| 7 | Circle |
| 9 | Land |
| 11 | Drift |
| 13 | Sport |
| 14 | Flip |
| 15 | AutoTune |
| 16 | PosHold |
| 17 | Brake |
| 18 | Throw |
| 19 | Avoid_ADSB |
| 20 | Guided_NoGPS |
| 21 | Smart_RTL |
| 22 | FlowHold |
| 23 | Follow |
| 24 | ZigZag |
| 25 | SystemID |
| 26 | Heli_Autorotate |
| 27 | Auto RTL |
| 28 | Turtle |

## 3. Clasificación MVP 0.1

Para MVP 0.1, `is_automatic_mode` debe clasificar exclusivamente ArduPilot Copter:

```text
if vehicle_family != ArduCopter:
    return Unknown

if custom_mode == 3:
    return Automatic

if custom_mode in known_arducopter_modes:
    return NotAutomatic

return Unknown
```

Regla conservadora:

- solo `custom_mode == 3` se considera `Automatic` en MVP 0.1;
- `Guided`, `RTL`, `Loiter`, `Land`, `Auto RTL` y otros modos automatizados o asistidos se clasifican como `NotAutomatic` para `ARM-AUTO-001`, pero deben quedar catalogados para reglas futuras;
- cualquier vehículo no Copter, modo desconocido o heartbeat no interpretable devuelve `Unknown`;
- en `Unknown`, aplica `CRITICAL-UNKNOWN-001`.

Esta decisión mantiene el alcance de la regla inicial estrecho y testeable. No pretende resolver todos los modos autónomos o guiados.

## 4. Observación en SITL

La validación debe registrar, como mínimo:

- versión de ArduPilot SITL;
- tipo de vehículo;
- secuencia de `HEARTBEAT`;
- `base_mode`;
- `custom_mode`;
- clasificación resultante: `Automatic`, `NotAutomatic` o `Unknown`;
- timestamp de observación;
- comando usado para cambiar a modo Auto en SITL, si aplica.

## 5. Procedimiento Reproducible

### Paso 1: Arrancar ArduPilot Copter SITL

Arrancar SITL sin hardware real y registrar versión y parámetros relevantes.

### Paso 2: Arrancar Gateway

Arrancar el gateway con logging de `flight_state` habilitado y política `unknown_mode_policy = "block"`.

### Paso 3: Observar HEARTBEAT Inicial

Registrar los primeros `HEARTBEAT` recibidos:

```text
HEARTBEAT custom_mode=<n> base_mode=<flags> autopilot=<autopilot> type=<type>
```

Criterio:

- si `custom_mode` corresponde a un modo Copter conocido distinto de `3`, la clasificación debe ser `NotAutomatic`;
- si no se puede interpretar, `Unknown`.

### Paso 4: Cambiar a Auto en SITL

Cambiar el modo del simulador a Auto usando mecanismo de SITL/GCS/MAVProxy en entorno controlado.

Criterio:

- el siguiente `HEARTBEAT` estable debe mostrar `custom_mode = 3`;
- el gateway debe registrar clasificación `Automatic`.

### Paso 5: Volver a un Modo No Automático

Cambiar a un modo Copter no Auto, por ejemplo Stabilize o Loiter en simulación.

Criterio:

- `custom_mode` debe cambiar a un valor conocido distinto de `3`;
- el gateway debe registrar clasificación `NotAutomatic`.

### Paso 6: Probar Modo Desconocido

Mediante test unitario o fixture, inyectar un `HEARTBEAT` con vehículo no soportado o `custom_mode` fuera de tabla.

Criterio:

- clasificación `Unknown`;
- comandos críticos posteriores caen bajo `CRITICAL-UNKNOWN-001`.

## 6. Tests Derivados

| ID | Entrada | Resultado esperado |
|---|---|---|
| MODE-AP-U-001 | ArduCopter `custom_mode=3` | `Automatic` |
| MODE-AP-U-002 | ArduCopter `custom_mode=0` | `NotAutomatic` |
| MODE-AP-U-003 | ArduCopter `custom_mode=5` | `NotAutomatic` |
| MODE-AP-U-004 | ArduCopter `custom_mode=27` | `NotAutomatic` en MVP 0.1 |
| MODE-AP-U-005 | Vehículo no soportado | `Unknown` |
| MODE-AP-U-006 | Sin `HEARTBEAT` válido | `Unknown` |
| MODE-AP-SITL-001 | Cambio SITL a Auto | log `flight_state.mode_changed` con `Automatic` |
| MODE-AP-SITL-002 | Cambio SITL fuera de Auto | log `flight_state.mode_changed` con `NotAutomatic` |

## 7. Logs Esperados

Evento:

```text
flight_state.mode_changed
```

Campos mínimos:

- `autopilot = ArduPilot`;
- `vehicle_family = ArduCopter`;
- `custom_mode`;
- `mode_name`;
- `classification`;
- `previous_classification`;
- `source_system`;
- `source_component`.

## 8. Riesgo Residual

- Modos como Guided, RTL, Land, SmartRTL o Auto RTL pueden tener comportamiento autónomo o semiautónomo, pero no se tratan como `Automatic` en MVP 0.1.
- Esta decisión reduce alcance, pero deja reglas futuras necesarias para cambio de modo, guided mode y control de misión.
- La clasificación debe revisarse antes de ampliar la política más allá de `ARM-AUTO-001`.

## 9. Fuentes

- ArduPilot Dev: Get and Set FlightMode.
- ArduPilot Copter: Flight Modes.
- ArduPilot Copter parameters `FLTMODE*`.

