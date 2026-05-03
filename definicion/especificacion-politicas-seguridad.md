# Especificación de Políticas de Seguridad

## 1. Objetivo

Definir cómo se expresan, evalúan y auditan las políticas de autorización del gateway. La política debe ser simple al inicio, testeable y conservadora.

## 2. Modelo de Decisión

Cada mensaje desde la GCS produce una decisión:

- `allow`: reenviar al vehículo.
- `block`: no reenviar.
- `audit_only`: reenviar, pero registrar que habría sido bloqueado.
- `drop_invalid`: descartar por error de parseo o validación.

## 3. Entradas de Política

- mensaje MAVLink parseado;
- `message_id`;
- comando MAVLink, si aplica;
- `system_id` y `component_id`;
- `target_system` y `target_component`, si existen;
- IP/puerto de origen para UDP;
- transporte usado;
- estado de vuelo conocido;
- configuración de allowlist;
- estado de signing/cifrado si se conoce.

## 4. Defaults Seguros

- Comandos críticos en estado desconocido: `block`.
- Orígenes UDP no esperados: `block` o `audit_only` solo si se configura expresamente.
- Configuración ambigua: error de arranque.
- Secretos ausentes con cifrado requerido: error de arranque.
- Logs de payload completo: deshabilitados por defecto.

## 5. Reglas Iniciales

### 5.1 ARM-AUTO-001

Bloquear armado no certificado en modo automático.

```text
if flight_mode == automatic
and command == MAV_CMD_COMPONENT_ARM_DISARM
and source_ip not in certified_ips
then block
```

Especificación testeable:

- `especificacion-arm-auto-001.md`

### 5.2 CRITICAL-UNKNOWN-001

Bloquear comandos críticos cuando el modo de vuelo sea desconocido.

```text
if flight_mode == unknown
and command in critical_commands
then block
```

### 5.3 PARSE-ERROR-001

Descartar paquetes que no puedan parsearse de forma segura.

```text
if mavlink_parse_result == error
then drop_invalid
```

### 5.4 Reglas de catálogo crítico fase 0015

Las siguientes reglas se aplican a tráfico UDP `GCS -> vehículo`. Si el modo de
vuelo es `Unknown`, prevalece `CRITICAL-UNKNOWN-001` para cualquier mensaje o
comando catalogado. Si el modo es conocido, estas reglas bloquean origen no
certificado y permiten origen certificado. Con `audit_only`, el gateway registra
que habría bloqueado y reenvía el datagrama.

| Regla | Mensajes/comandos cubiertos | Acción inicial |
|---|---|---|
| `PARAM-SET-001` | `PARAM_SET` | Bloquear IP no certificada |
| `MISSION-UPLOAD-001` | `MISSION_COUNT`, `MISSION_ITEM`, `MISSION_ITEM_INT`, `MISSION_CLEAR_ALL`, `MISSION_SET_CURRENT`, `MISSION_WRITE_PARTIAL_LIST`, `MAV_CMD_MISSION_START` | Bloquear IP no certificada |
| `MODE-CHANGE-001` | `SET_MODE`, `MAV_CMD_DO_SET_MODE` | Bloquear IP no certificada |
| `NAV-MOVEMENT-001` | `MAV_CMD_NAV_TAKEOFF`, `MAV_CMD_NAV_LAND` | Bloquear IP no certificada |
| `GUIDED-REPOSITION-001` | `MAV_CMD_DO_REPOSITION` | Bloquear IP no certificada |
| `RC-OVERRIDE-001` | `MANUAL_CONTROL`, `RC_CHANNELS_OVERRIDE` | Bloquear IP no certificada |
| `PREFLIGHT-REBOOT-001` | `MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN` | Bloquear IP no certificada |

### 5.5 NAV-MOVEMENT-001

Bloquear comandos directos de despegue o aterrizaje desde IP no certificada
cuando el modo de vuelo es conocido.

```text
if direction == GCS_TO_VEHICLE
and command in [MAV_CMD_NAV_TAKEOFF, MAV_CMD_NAV_LAND]
and flight_mode != unknown
and source_ip not in certified_ips
then block
```

Notas:

- Si el modo de vuelo es `Unknown`, prevalece `CRITICAL-UNKNOWN-001`.
- Si `signing.policy = "enforce"`, takeoff/land requieren firma válida porque
  siguen clasificados como `Critical`.
- Esta regla cubre la ruta de comando directo parseada por el gateway; rutas de
  misión o dialecto específico quedan sujetas al catálogo y a fases posteriores.

## 6. Eventos de Auditoría

Todo evento de política debe incluir:

- timestamp;
- decisión;
- regla;
- transporte;
- origen no sensible;
- `message_id`;
- comando si aplica;
- modo de vuelo conocido;
- `target_system` y `target_component` si existen;
- motivo humano-legible;
- id de correlación si existe.

No debe incluir:

- claves;
- nonces secretos si el diseño los considera sensibles;
- payload completo en producción;
- datos innecesarios de localización en logs normales.

## 7. Evolución de Políticas

Las siguientes reglas deben considerarse tras la primera versión:

- antireplay con signing o canal autenticado;
- políticas específicas por autopiloto.

## 8. Pruebas Mínimas

- Comando crítico permitido desde IP certificada.
- Comando crítico bloqueado desde IP no certificada.
- Comando crítico bloqueado con modo desconocido.
- Mensaje no crítico permitido.
- Paquete inválido descartado.
- `audit_only` registra sin bloquear.
