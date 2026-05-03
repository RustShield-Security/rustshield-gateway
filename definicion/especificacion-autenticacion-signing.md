# Especificación de Autenticación y MAVLink Signing

## 1. Objetivo

Definir la política futura para distinguir observación de paquetes firmados, autenticación criptográfica validada y rechazo de comandos no autenticados.

## 2. Alcance

Incluye:

- estados `authenticated=true/false`;
- modos de política `observe`, `audit` y `enforce`;
- eventos, métricas y fixtures necesarios;
- requisitos de claves y timestamp;
- compatibilidad SITL con QGroundControl y ArduPilot.

No incluye:

- implementación criptográfica en la fase 0011;
- activación de signing obligatorio fuera del laboratorio controlado;
- hardware real;
- cifrado externo hacia QGroundControl estándar.

## 3. Estados

| Estado | Significado | Permitido en fase actual |
|---|---|---|
| `signed_observed=true` | El frame MAVLink 2 tiene flag de signing. | Sí |
| `authenticated=false` | No hay validación criptográfica completa o la validación falla. | Sí |
| `authenticated=true` | Firma válida, timestamp aceptado y clave/política del enlace coinciden. | Sí, en `audit` y `enforce` con clave configurada |

Regla de seguridad: ningún log, métrica o decisión debe inferir autenticación desde IP, puerto, `system_id`, `component_id` o presencia del flag de signing.

## 4. Modos de Política

### 4.1 `observe`

Modo compatible por defecto.

- Paquetes firmados: observar y reenviar si la política existente lo permite.
- Paquetes no firmados: reenviar si la política existente lo permite.
- Comandos críticos: siguen sujetos a `ARM-AUTO-001` y `CRITICAL-UNKNOWN-001`.
- `authenticated` siempre queda en `false`.

### 4.2 `audit`

Modo de migración.

- Paquetes firmados válidos: reenviar y registrar `authenticated=true`.
- Paquetes firmados inválidos: warning y decisión según política de enlace.
- Comandos críticos sin firma: warning obligatorio; bloqueo configurable.
- No debe usarse como claim de seguridad fuerte sin evidencias de validación.

### 4.3 `enforce`

Modo de autenticación requerida.

- Comandos críticos GCS -> vehículo sin firma válida: bloquear.
- Firma inválida: bloquear en GCS -> vehículo.
- Replay detectado: bloquear.
- Telemetría vehículo -> GCS puede tener política separada, documentada por enlace.

Decisión fase 0018 / ADR 0010:

- `enforce` se limita inicialmente a comandos catalogados como críticos o de
  alto riesgo en dirección GCS -> vehículo;
- telemetría vehículo -> GCS no firmada se reenvía por defecto si el resto de la
  política la permite;
- telemetría vehículo -> GCS con firma inválida o replay se audita y contabiliza
  sin bloqueo en esta fase;
- `security.audit_only` no puede convertir un rechazo criptográfico de
  `enforce` en reenvío;
- comandos críticos autenticados siguen sujetos a la política semántica del
  gateway;
- `SETUP_SIGNING` no se reenvía automáticamente entre enlaces.

## 5. Gestión de Claves

- Las claves de signing son material secreto.
- No se guardan en documentos versionados.
- No se exponen por parámetros MAVLink, logs, crash dumps ni métricas.
- La carga de claves debe requerir configuración explícita y validación de permisos.
- `SETUP_SIGNING` no se debe reenviar automáticamente entre enlaces.
- Cualquier procedimiento de provisionamiento debe asumir canal seguro separado.
- `enforce` debe fallar al arrancar si falta clave o si el archivo de clave no
  cumple las validaciones de permisos.
- Para laboratorio, `audit` y `enforce` cargan claves desde un archivo local de
  32 bytes en hexadecimal. La ruta debe ser absoluta, regular, no symlink, fuera
  del worktree Git actual del gateway y, en Unix, propiedad del usuario efectivo
  del proceso con permisos sin acceso de grupo ni mundo.
- La recuperación de laboratorio ante clave perdida, rotación fallida o
  timestamp desincronizado requiere parada controlada, cambio explícito de
  configuración a `audit`/`observe` o resincronización del emisor firmante, y
  reinicio documentado.
- No se admite fail-open automático ni bypass por configuración de
  `security.audit_only`.

## 6. Timestamp y Replay

La validación futura debe mantener el último timestamp aceptado por:

```text
(SystemID, ComponentID, LinkID)
```

Un paquete en modo `enforce` debe rechazarse si:

- la firma no coincide;
- el timestamp retrocede respecto al último aceptado para el stream;
- el timestamp es demasiado antiguo respecto a la ventana configurada;
- el `linkID` no está permitido para el enlace.

## 7. Eventos

Actuales:

- `mavlink.signed_observed`
- `mavlink.signing_validated`
- `mavlink.signing_rejected`

Futuros:

- `mavlink.setup_signing_observed`

Campos mínimos:

- `event`;
- `direction`;
- `message_id`;
- `system_id`;
- `component_id`;
- `link_id` cuando esté disponible;
- `authenticated`;
- `signing_policy`;
- `reason` cuando haya rechazo.

Campos prohibidos:

- clave;
- firma completa;
- payload completo por defecto;
- contenido de `SETUP_SIGNING`.

## 8. Métricas

Actuales:

- `packets_signed_observed_total`
- `packets_signed_valid_total`
- `packets_signed_invalid_total`
- `packets_unsigned_rejected_total`
- `signing_replay_rejected_total`

Futuras:

- `setup_signing_observed_total`

## 9. Fixtures y Tests

| Fixture | Propósito |
|---|---|
| `mavlink2_unsigned_heartbeat` | Control no firmado permitido. |
| `mavlink2_signed_heartbeat_valid` | Validación positiva de firma y timestamp. |
| `mavlink2_signed_heartbeat_bad_signature` | Rechazo por firma inválida. |
| `mavlink2_signed_command_arm_valid` | Comando crítico autenticado. |
| `mavlink2_unsigned_command_arm_enforce` | Bloqueo de comando crítico sin firma. |
| `mavlink2_signed_command_bad_signature_enforce` | Bloqueo de comando crítico con firma inválida. |
| `mavlink2_signed_command_replay` | Rechazo por timestamp regresivo. |
| `mavlink2_unsigned_telemetry_enforce` | Telemetría sin firma reenviada en `enforce`. |
| `mavlink2_signed_telemetry_bad_signature_enforce` | Telemetría con firma inválida auditada sin bloqueo en fase 0019. |
| `setup_signing_direct` | Tratamiento sensible de provisionamiento. |
| `setup_signing_broadcast` | Rechazo o bloqueo de reenvío automático. |

Estado fase 0012:

- fixtures deterministas implementados para heartbeat firmado válido, heartbeat
  con firma alterada, replay de paquete firmado y comando ARM firmado válido;
- validación inicial implementada solo como módulo/test, no conectada al
  pipeline operativo UDP;
- `authenticated=true` sigue sin emitirse en runtime operativo.

Estado fase 0018:

- ADR 0010 acepta `enforce` solo para comandos críticos/high-risk GCS ->
  vehículo en laboratorio controlado;
- la implementación de 0019 debe conservar política separada para telemetría y
  no activar hardware real ni gestión operacional de claves.

Estado fase 0019:

- `signing.policy = "enforce"` queda implementado en runtime para comandos
  críticos/high-risk GCS -> vehículo;
- comandos críticos/high-risk sin firma, con firma inválida o replay se
  bloquean antes de `security.audit_only`;
- telemetría vehículo -> GCS se mantiene disponible y auditada en `enforce`.

Estado fase 0020:

- gestión de clave local de laboratorio documentada en
  `implementacion/procedimientos/gestion-claves-signing-laboratorio.md`;
- validación reforzada de ubicación, symlink, permisos y propietario;
- KMS/HSM queda como diseño futuro sujeto a ADR, sin backend implementado por
  defecto.

## 10. Compatibilidad

- QGroundControl sigue siendo GCS primaria para el flujo transparente.
- ArduPilot Copter SITL sigue siendo autopiloto primario de simulación.
- ArduPilot documenta que signing no cifra telemetría y afecta respuesta a comandos; por tanto, la política del gateway debe separar telemetría de comandos.
- Mission Planner aparece en documentación ArduPilot como herramienta con soporte de configuración de signing; se puede usar en laboratorio futuro, pero no sustituye a QGroundControl como GCS primaria sin nueva fase.
