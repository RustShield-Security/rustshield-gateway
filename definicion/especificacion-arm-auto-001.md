# Especificación Testeable: ARM-AUTO-001

## 1. Objetivo

Convertir la regla `ARM-AUTO-001` en una especificación verificable para MVP 0.1 usando ArduPilot SITL como autopiloto primario.

La regla evita que un comando de armado no certificado llegue al vehículo cuando el estado conocido indica modo automático. La validación inicial debe hacerse solo en simulación.

## 2. Fuentes Técnicas

La documentación de ArduPilot describe `MAV_CMD_COMPONENT_ARM_DISARM` dentro de `COMMAND_LONG` con:

- `command = MAV_CMD_COMPONENT_ARM_DISARM = 400`;
- `param1 = 1` para armar;
- `param1 = 0` para desarmar;
- `param2 = 21196` para forzar armado o desarmado, intentando saltar comprobaciones normales.

MVP 0.1 debe tratar el armado normal y el armado forzado como críticos. El desarmado también es crítico, pero la primera regla se centra en impedir armado no autorizado.

## 3. Alcance MVP 0.1

Autopiloto:

- ArduPilot SITL.

Mensajes inspeccionados:

- `COMMAND_LONG`;
- `COMMAND_INT` si el crate/dialecto expone `command` de forma equivalente;
- `HEARTBEAT` para estado.

Comando crítico:

- `MAV_CMD_COMPONENT_ARM_DISARM`.

Transporte:

- UDP.

Identidad inicial:

- IP de origen incluida en `certified_ips`.
- Esta identidad es débil y no sustituye MAVLink signing ni autenticación fuerte.

## 4. Definición de Modo Automático

Para MVP 0.1, el gateway debe implementar una función explícita:

```text
is_automatic_mode(autopilot, base_mode, custom_mode) -> Automatic | NotAutomatic | Unknown
```

Reglas:

- Si `autopilot` corresponde a ArduPilot y el mapping de `custom_mode` está implementado y validado, devolver `Automatic` o `NotAutomatic`.
- Si no hay `HEARTBEAT`, devolver `Unknown`.
- Si el autopiloto no es ArduPilot o el modo no puede interpretarse, devolver `Unknown`.
- En `Unknown`, aplicar `CRITICAL-UNKNOWN-001`.

Nota: el mapping exacto de `custom_mode` debe validarse con ArduPilot SITL antes de marcar la regla como cerrada.

## 5. Detección de Armado

Un mensaje debe clasificarse como intento de armado si:

```text
message in [COMMAND_LONG, COMMAND_INT]
and command == MAV_CMD_COMPONENT_ARM_DISARM
and param1 == 1
```

Un mensaje debe clasificarse como intento de armado forzado si además:

```text
param2 == 21196
```

Política:

- armado normal no certificado en automático: `block`;
- armado forzado no certificado en automático: `block` con severidad mayor;
- armado en modo desconocido: `block`;
- armado desde IP certificada: `allow` solo si el resto de validaciones pasa;
- desarmado: crítico, pero política específica pendiente.

## 6. Regla Formal

```text
if source_direction == gcs_to_vehicle
and transport == udp
and command == MAV_CMD_COMPONENT_ARM_DISARM
and param1 == 1
and flight_mode == automatic
and source_ip not in certified_ips
then decision = block
```

## 7. Evento de Auditoría Esperado

Campos mínimos:

- `event = security.command_blocked`;
- `rule_id = ARM-AUTO-001`;
- `decision = block`;
- `transport = udp`;
- `source_ip`;
- `message_id`;
- `command = MAV_CMD_COMPONENT_ARM_DISARM`;
- `param1_class = arm`;
- `param2_class = normal | force`;
- `flight_mode = automatic`;
- `autopilot = ArduPilot`;
- `reason`.

## 8. Tests Unitarios

| ID | Caso | Entrada | Resultado |
|---|---|---|---|
| ARM-AUTO-U-001 | Armado no certificado en automático | `param1=1`, IP fuera de allowlist | `block` |
| ARM-AUTO-U-002 | Armado certificado en automático | `param1=1`, IP en allowlist | `allow` |
| ARM-AUTO-U-003 | Armado forzado no certificado | `param1=1`, `param2=21196` | `block`, severidad alta |
| ARM-AUTO-U-004 | Armado con modo desconocido | sin `HEARTBEAT` válido | `block` por `CRITICAL-UNKNOWN-001` |
| ARM-AUTO-U-005 | Mensaje no crítico | otro comando | `allow` salvo otra regla |
| ARM-AUTO-U-006 | Paquete inválido | parse error | `drop_invalid` |

## 9. Prueba SITL

La prueba SITL debe demostrar:

- el gateway observa `HEARTBEAT`;
- el modo se clasifica como automático o desconocido de forma explícita;
- un comando de armado no certificado no llega a ArduPilot SITL;
- el evento queda registrado;
- la métrica de bloqueo aumenta.

No se debe ejecutar esta prueba contra hardware real.

## 10. Preguntas Pendientes

- Mapping exacto de modos ArduPilot a partir de `custom_mode`.
- Política para desarmado, especialmente en vuelo.
- Política para `MAV_CMD_COMPONENT_ARM_DISARM` con `target_system=0`.
- Tratamiento de comandos equivalentes en dialectos específicos.

