# Trazabilidad MVP 0.1

## 1. Objetivo

Mantener una tabla explícita entre requisitos, amenazas, reglas, tests, logs y métricas del MVP 0.1. Esta trazabilidad es obligatoria para evitar que una regla de seguridad se implemente sin evidencia observable.

## 2. Tabla Principal

| Requisito | Amenaza | Regla / decisión | Test | Log esperado | Métrica |
|---|---|---|---|---|---|
| PRD objetivo 1: proxy MAVLink UDP bidireccional | T-004, T-010 | Proxy UDP transparente | SITL-001, integración UDP | `transport.opened`, `gateway.start` | `packets_received_total`, `packets_forwarded_total` |
| PRD objetivo 2: separar telemetría, comandos y estado | T-005 | Pipeline `flight_state` + `security_filter` | SITL-002, MODE-AP-U-* | `flight_state.mode_changed` | `last_heartbeat_age_ms` |
| PRD objetivo 3: bloquear armado no certificado en Auto | T-001, T-002 | `ARM-AUTO-001` | ARM-AUTO-U-001, SITL-003 | `security.command_blocked` | `packets_blocked_total`, `commands_critical_observed_total` |
| Política ante modo desconocido | T-012 | `CRITICAL-UNKNOWN-001` | ARM-AUTO-U-004, MODE-AP-U-006 | `security.command_blocked` | `packets_blocked_total` |
| Catálogo crítico fase 0015: parámetros, misión, modo, reposition, RC/manual y reboot | T-001, T-002, T-003, T-012 | `PARAM-SET-001`, `MISSION-UPLOAD-001`, `MODE-CHANGE-001`, `GUIDED-REPOSITION-001`, `RC-OVERRIDE-001`, `PREFLIGHT-REBOOT-001` | tests `security_filter::*_001_*`, `mavlink_codec::parses_critical_message_fixtures_into_security_commands` | `security.command_observed`, `security.command_blocked`, `security.audit_only` | `commands_critical_observed_total`, `packets_blocked_total`, `policy_latency_us` |
| Parseo robusto de input no confiable | T-004 | `PARSE-ERROR-001` | ARM-AUTO-U-006, SITL-006, fuzz parser | `mavlink.parse_error` | `packets_parse_error_total` |
| IP allowlist como control débil | T-002, T-003 | ADR 0006 | ARM-AUTO-U-001/U-002 | `security.command_observed`, `security.command_blocked` | `commands_critical_observed_total` |
| No romper routing ni signing | T-005, T-006 | ADR 0007, ADR 0008 | test transparencia bytes/campos, `signed_packets_are_observed_without_claiming_authentication` | `security.command_observed`, `mavlink.signed_observed` con `authenticated=false` | `packets_forwarded_total`, `packets_signed_observed_total` |
| Autenticación fuerte para comandos críticos/high-risk | T-002, T-003, T-006 | ADR 0009; ADR 0010; `enforce` limitado a GCS -> vehículo | tests `signing::*`; `transport::signing_enforce_*` para firma válida, sin firma, firma inválida, replay, no bypass por `audit_only` y telemetría disponible | `mavlink.signing_validated`, `mavlink.signing_rejected`, `mavlink.signed_observed` | `packets_signed_valid_total`, `packets_signed_invalid_total`, `packets_unsigned_rejected_total`, `signing_replay_rejected_total` |
| Cifrado no operativo con QGC estándar | T-009 | ADR 0004 | test harness crypto | `crypto.error` si configuración inválida | `packets_crypto_error_total` |
| No filtrar secretos en logs | T-007, T-011 | Observabilidad + gestión de claves | revisión logs, tests redacción | ausencia de campos secretos | N/A o contador de errores de config |
| Latencia interna objetivo | R-008 | Benchmark MVP 0.1 | B-001, B-002 | resumen benchmark | `processing_latency_ms`, `parse_latency_ms`, `policy_latency_ms` |

## 3. Trazabilidad de ARM-AUTO-001

| Campo | Valor |
|---|---|
| Requisito | PRD objetivo 3; `especificacion-arm-auto-001.md` |
| Amenaza | T-001: inyección de comando de armado; T-002: spoofing UDP |
| Regla | `ARM-AUTO-001` |
| Condición | GCS -> vehículo, UDP, `MAV_CMD_COMPONENT_ARM_DISARM`, `param1=1`, modo `Automatic`, IP no certificada |
| Acción | `block`, no reenviar al vehículo |
| Tests | ARM-AUTO-U-001, ARM-AUTO-U-003, SITL-003 |
| Log | `security.command_blocked` |
| Métricas | `packets_blocked_total`, `commands_critical_observed_total` |
| Riesgo residual | IP allowlist no es autenticación fuerte; replay no mitigado sin signing validado |

## 4. Trazabilidad de Modo ArduPilot

| Campo | Valor |
|---|---|
| Requisito | `validacion-modo-ardupilot-sitl.md` |
| Amenaza | T-012: política insegura ante estado desconocido |
| Regla | `is_automatic_mode` para ArduCopter |
| Condición | `HEARTBEAT.custom_mode == 3` |
| Acción | clasificar como `Automatic` |
| Tests | MODE-AP-U-001, MODE-AP-SITL-001 |
| Log | `flight_state.mode_changed` |
| Métrica | `last_heartbeat_age_ms` |
| Riesgo residual | Modos autónomos distintos de Auto quedan para reglas futuras |

## 5. Criterio de Mantenimiento

Cada nueva regla o decisión del MVP debe añadir una fila a esta tabla antes de implementarse. Si una amenaza no tiene test, log o métrica, no está suficientemente cubierta.
