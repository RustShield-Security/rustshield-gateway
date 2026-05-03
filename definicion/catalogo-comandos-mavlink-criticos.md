# Catálogo de Comandos MAVLink Críticos

## 1. Objetivo

Catalogar mensajes y comandos MAVLink que pueden afectar seguridad operacional, integridad de misión, estado del vehículo o control directo. Este catálogo guía el filtro de seguridad, las pruebas y las decisiones de política.

## 2. Criterios de Criticidad

Un mensaje o comando se considera crítico si puede:

- armar o desarmar;
- cambiar modo de vuelo;
- iniciar despegue o aterrizaje;
- modificar misión;
- modificar parámetros;
- alterar failsafe, geofence o home;
- introducir control manual o guiado;
- reiniciar componentes;
- degradar navegación o seguridad.

## 3. Escala de Severidad

| Severidad | Significado | Ejemplos |
|---|---|---|
| Crítica | Puede iniciar movimiento, armar, alterar control directo o comprometer seguridad inmediata. | Armado, takeoff, guided, RC override |
| Alta | Puede alterar misión, parámetros, navegación, failsafe o estado operativo relevante. | Mission upload, `PARAM_SET`, geofence |
| Media | Puede afectar observabilidad, carga, routing o comportamiento no inmediato. | Stream rates, peticiones de datos |
| Baja | Telemetría o mensajes sin acción directa conocida. | Estado, información pasiva |

## 4. Mensajes MAVLink Prioritarios

| Mensaje | Uso | Criticidad | Tratamiento inicial |
|---|---|---:|---|
| `HEARTBEAT` | Estado, modo, armado | Alta para estado | Parsear y actualizar `flight_state` |
| `COMMAND_LONG` | Comandos generales | Crítica | Clasificar y filtrar |
| `COMMAND_INT` | Comandos con coordenadas enteras | Crítica | Clasificar y filtrar |
| `MISSION_COUNT` / `MISSION_ITEM_INT` | Carga de misión | Alta | `MISSION-UPLOAD-001` |
| `PARAM_SET` | Mutación de parámetros | Alta | `PARAM-SET-001` |
| `SET_MODE` | Cambio de modo | Crítica | `MODE-CHANGE-001` |
| `MANUAL_CONTROL` | Control manual | Crítica | `RC-OVERRIDE-001` |
| `RC_CHANNELS_OVERRIDE` | Override RC | Crítica | `RC-OVERRIDE-001` |
| `SETUP_SIGNING` | Provisionamiento de clave MAVLink signing | Sensible | No reenviar automáticamente; no registrar payload, clave ni firma completa |

## 5. Comandos Críticos Iniciales

| Comando | Mensaje portador | Dirección esperada | Parámetros sensibles | Severidad | Política MVP 0.1 |
|---|---|---|---|---:|---|
| `MAV_CMD_COMPONENT_ARM_DISARM` | `COMMAND_LONG`, `COMMAND_INT` si aplica | GCS -> vehículo | `param1=1` arma, `param1=0` desarma, `param2=21196` fuerza | Crítica | `ARM-AUTO-001` y `CRITICAL-UNKNOWN-001` |
| `MAV_CMD_NAV_TAKEOFF` | `COMMAND_LONG`/misión | GCS -> vehículo | altitud y posición según dialecto | Crítica | `NAV-MOVEMENT-001`, `CRITICAL-UNKNOWN-001`, `enforce` |
| `MAV_CMD_NAV_LAND` | `COMMAND_LONG`/misión | GCS -> vehículo | ubicación/abort según contexto | Crítica | `NAV-MOVEMENT-001`, `CRITICAL-UNKNOWN-001`, `enforce` |
| `MAV_CMD_DO_SET_MODE` | `COMMAND_LONG` | GCS -> vehículo | modo objetivo | Crítica | `MODE-CHANGE-001` |
| `MAV_CMD_MISSION_START` | `COMMAND_LONG` | GCS -> vehículo | índices de misión | Alta | `MISSION-UPLOAD-001` |
| `MAV_CMD_DO_REPOSITION` | `COMMAND_INT`/`COMMAND_LONG` | GCS -> vehículo | coordenadas, velocidad, flags | Crítica | `GUIDED-REPOSITION-001` |
| `MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN` | `COMMAND_LONG` | GCS -> vehículo | componente y acción | Alta | `PREFLIGHT-REBOOT-001` |

## 6. Regla Inicial: ARM-AUTO-001

Condición:

- `flight_mode == automatic`;
- comando detectado: `MAV_CMD_COMPONENT_ARM_DISARM`;
- origen no certificado.

Acción:

- bloquear;
- no reenviar;
- registrar evento;
- incrementar métrica.

Política ante estado desconocido:

- bloquear comandos críticos por defecto hasta recibir estado válido.

Documento específico:

- `especificacion-arm-auto-001.md`

## 7. Campos a Inspeccionar

Para `COMMAND_LONG` y `COMMAND_INT`:

- `command`;
- `target_system`;
- `target_component`;
- parámetros relevantes del comando;
- `system_id` y `component_id` de origen cuando estén disponibles en el frame;
- IP y puerto de origen si el transporte es UDP.

Para `HEARTBEAT`:

- `type`;
- `autopilot`;
- `base_mode`;
- `custom_mode`;
- `system_status`.

## 8. Política de Cobertura MVP 0.1

La cobertura implementada tras la fase 0015 es:

- detectar y bloquear armado no certificado en modo automático;
- bloquear comandos críticos catalogados cuando el estado sea desconocido;
- bloquear desde IP no certificada mutaciones de parámetros, carga/cambio de
  misión, cambios de modo, takeoff/land, reposition/guided, control manual/RC
  override y reboot/shutdown preflight;
- bloquear `SETUP_SIGNING` como mensaje sensible que no se reenvía
  automáticamente entre enlaces;
- exigir firma válida con `signing.policy = "enforce"` para comandos
  catalogados como críticos o de alto riesgo en tráfico GCS -> vehículo;
- registrar comandos críticos observados aunque no tengan regla activa;
- mantener `audit_only` como modo de validación sin bloqueo efectivo;
- no afirmar cobertura completa de todos los dialectos, autopilotos o comandos
  críticos posibles.

## 9. Backlog de Reglas Posteriores

| Regla candidata | Motivo | Prioridad |
|---|---|---:|
| `PARAM-SET-001` | Mutación persistente de configuración | Alta |
| `MISSION-UPLOAD-001` | Cambio de misión | Alta |
| `MODE-CHANGE-001` | Cambio de modo de vuelo | Alta |
| `NAV-MOVEMENT-001` | Despegue o aterrizaje por comando directo | Alta |
| `RC-OVERRIDE-001` | Control manual externo | Alta |
| `GUIDED-REPOSITION-001` | Reposición o control guiado | Alta |
| `PREFLIGHT-REBOOT-001` | Reinicio o apagado de componente | Media |

Estado tras fases 0015 y 0021C-2A: las reglas anteriores quedan implementadas
como primer catálogo operativo. `SETUP_SIGNING` queda tratado como mensaje
sensible bloqueado por el transporte, no como comando de control de vuelo. La
evolución posterior debe ampliar cobertura por dialecto, autopiloto y fixtures
SITL antes de hacer claims de protección completa.

## 10. Limitaciones

- La interpretación exacta de modos puede variar entre ArduPilot y PX4.
- Algunos comandos críticos pueden viajar en mensajes distintos según autopiloto o GCS.
- La primera versión no debe pretender cobertura completa.
- Este catálogo debe versionarse y crecer con pruebas SITL.
