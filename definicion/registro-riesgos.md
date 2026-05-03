# Registro de Riesgos

## 1. Objetivo

Mantener una lista viva de riesgos técnicos, de seguridad, compatibilidad y operación. Cada riesgo debe tener mitigación y propietario documental.

## 2. Riesgos Iniciales

| ID | Riesgo | Impacto | Probabilidad | Mitigación | Estado |
|---|---|---:|---:|---|---|
| R-001 | Cifrado rompe compatibilidad con GCS estándar | Alto | Alta | ADR 0004 y matriz de compatibilidad; cifrado limitado a harness aislado en MVP 0.1 | Residual post-MVP |
| R-002 | IP allowlist se interpreta como autenticación fuerte | Alto | Media | ADR 0006, ADR 0009 y ADR 0010; `enforce` solo acepta `authenticated=true` con firma/timestamp/link_id válidos para comandos críticos | Mitigado en laboratorio para comandos críticos / residual operación |
| R-003 | Modo automático mal inferido | Alto | Media | Validación ArduPilot Copter SITL; `custom_mode == 3` como único `Automatic` de MVP 0.1 | Mitigado para alcance MVP / residual post-MVP |
| R-004 | Reutilización de nonce AEAD | Crítico | Media | Especificación de claves/nonces y tests; cifrado no operativo en MVP 0.1 | Residual post-MVP |
| R-005 | Parser cae por tráfico malformado | Alto | Media | Manejo `PARSE-ERROR-001`, tests y fuzzing inicial sin crash | Mitigado inicialmente / residual post-MVP |
| R-006 | Routing MAVLink alterado | Alto | Media | Modo transparente por defecto, proxy UDP bidireccional y tests de no reserialización intencional | Mitigado inicialmente / residual post-MVP |
| R-007 | Signing MAVLink roto | Alto | Media | ADR 0008, ADR 0009 y ADR 0010; transparencia por defecto, `enforce` limitado a comandos críticos, tests de no reserialización y telemetría separada | Mitigado en laboratorio / residual compatibilidad SITL y claves |
| R-008 | Latencia supera 1 ms | Medio | Media | Benchmark interno y evidencia SITL/QGroundControl con procesamiento interno max sub-ms; límites externos documentados | Mitigado para latencia interna / residual end-to-end externa |
| R-009 | Logs exponen información sensible | Medio | Media | Tests de redacción y ausencia de payload completo por defecto | Mitigado inicialmente / residual revisión continua |
| R-010 | Serial real se usa antes de validación segura | Alto | Baja | Política de seguridad, skill de proyecto y fases limitadas a SITL/UDP | Mitigado por alcance / abierto para hardware real |
| R-011 | Catálogo de comandos críticos incompleto o interpretado como cobertura total | Alto | Media | Fase 0015 cubre un catálogo inicial con tests y documentación explícita de límites; no se afirma cobertura completa por dialecto/autopiloto | Mitigado inicialmente / residual post-MVP |

## 3. Reglas de Gestión

- Todo riesgo alto debe tener documento asociado o ADR.
- Todo riesgo que cambie arquitectura debe generar ADR.
- Riesgos cerrados deben conservar evidencia de validación.
- Riesgos de hardware real no se cierran con pruebas SITL.

## 4. Riesgos Residuales tras Cierre MVP 0.1

El MVP 0.1 queda cerrado para UDP + SITL + QGroundControl + ArduPilot Copter SITL, pero no elimina los riesgos que dependen de autenticación fuerte, hardware real, campañas prolongadas o compatibilidad con más autopilotos/GCS.

Riesgos prioritarios para la fase post-MVP:

- R-002 y R-007: definir una ruta de autenticación/signing que no rompa transparencia ni compatibilidad.
- R-005 y R-006: ampliar fuzzing y fixtures de routing para tráfico MAVLink más variado.
- R-008: mantener evidencia de latencia con límites explícitos y añadir
  timestamps correlacionables si se quiere medir GCS/simulador/red.
- R-003: ampliar matriz de modos solo cuando el alcance incluya modos/autopilotos adicionales.
- R-010: mantener hardware real fuera de operación hasta que exista fase, procedimiento y aceptación explícita.

## 5. Priorización para Iteración Post-MVP

| Prioridad | Riesgos | Motivo | Acción Recomendada |
|---:|---|---|---|
| P1 | R-002, R-007 | La autenticación/signing define el salto de control débil por IP a control criptográfico real. | Preparar ADR de autenticación/signing y mantener `mavlink.signed_observed` como evidencia sin claim de autenticación. |
| P2 | R-005, R-006 | Parser y routing son frontera directa con input no confiable y compatibilidad GCS/autopiloto. | Ampliar fuzzing, corpus y fixtures de routing transparente antes de añadir más políticas. |
| P3 | R-008 | La latencia interna está documentada, pero no mide GCS, simulador, red ni scheduling externo. | Añadir harness con timestamps correlacionables si el producto necesita claim end-to-end externo. |
| P4 | R-003 | La clasificación de modos está contenida al alcance ArduPilot Copter SITL. | No ampliar modos hasta definir nuevo alcance y tests SITL específicos. |
| P5 | R-010 | Hardware real queda fuera del modelo validado. | Mantener bloqueo documental y operativo hasta fase explícita. |

## 6. Actualización tras ADR 0009

R-002 y R-007 pasan de riesgo residual genérico a diseño de mitigación. La mitigación todavía no está implementada: no existe `authenticated=true` operativo, no hay gestión de claves activa y no se exige signing. La siguiente reducción de riesgo requiere fixtures firmados/no firmados, validación criptográfica, control de timestamp por `(SystemID, ComponentID, LinkID)` y pruebas SITL.

## 7. Actualización tras Fase 0012

La fase 0012 introduce validación inicial de firmas solo en tests, con clave
insegura de prueba y fixtures deterministas. Esto reduce incertidumbre técnica
sobre extracción y verificación, pero R-002 y R-007 no se consideran mitigados
operativamente hasta integrar política `audit/enforce`, gestión de claves,
eventos de runtime y validación SITL.

## 8. Actualización tras Fase 0016

R-005 y R-006 reducen incertidumbre con corpus determinista ampliado, campaña
`cargo fuzz` corta sin crash lógico y tests de transparencia de bytes para
MAVLink v1, MAVLink v2 y paquetes firmados permitidos. El riesgo residual sigue
abierto: la campaña no sustituye fuzzing prolongado, pruebas con más dialectos ni
validación continua de routing con GCS/autopiloto reales.

## 9. Actualización tras Fase 0017

R-008 queda mitigado para latencia interna del gateway en el alcance
SITL/QGroundControl documentado: la evidencia
`implementacion/evidencias/latency-e2e-sitl-20260502T143442Z/` registra
`processing_avg_us=60.769` y `processing_latency_max_us=246` desde
`metrics.snapshot`. El riesgo residual permanece para claims end-to-end externos
porque la medición no incluye UI de QGroundControl, scheduling del sistema
operativo, red física, Serial, simulador ni control-loop del autopiloto.

## 10. Actualización tras ADR 0010

R-002 y R-007 quedan diseñados para mitigación operativa de laboratorio, pero no
cerrados: `definicion/adr/0010-enforce-signing-comandos-criticos.md` autoriza
implementar `enforce` solo para comandos críticos/high-risk GCS -> vehículo con
firma, timestamp y `link_id` válidos. La telemetría mantiene política separada
de disponibilidad y se audita sin bloqueo en esta fase. El riesgo residual
permanece hasta que la fase 0019 implemente tests, métricas, bloqueo de
unsigned/invalid/replay y validación SITL/QGroundControl.

## 11. Actualización tras Fase 0019

R-002 y R-007 quedan mitigados en laboratorio para comandos críticos/high-risk
GCS -> vehículo: `signing.policy = "enforce"` exige clave local y bloquea
comandos sin firma, con firma inválida o replay antes de `security.audit_only`.
La mitigación está cubierta por tests unitarios/transporte y métricas
`packets_unsigned_rejected_total`, `packets_signed_invalid_total` y
`signing_replay_rejected_total`. El riesgo residual sigue abierto para gestión
operacional de claves, rotación, validación SITL/QGroundControl de `enforce`,
telemetría autenticada y hardware real.
