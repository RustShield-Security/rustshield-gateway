# Definition of Done

## 1. Objetivo

Definir cuándo una pieza de documentación, arquitectura o código del gateway puede considerarse terminada con rigor suficiente.

## 2. Documentos

Un documento está listo cuando:

- declara objetivo, alcance y no alcance;
- explicita supuestos;
- enumera riesgos y preguntas abiertas;
- referencia documentos relacionados;
- contiene criterios de validación;
- no contradice la política de seguridad del proyecto;
- declara su relación con MVP 0.1 cuando aplique;
- distingue entre SITL, laboratorio y hardware real;
- identifica si requiere fuente oficial antes de implementación.

## 3. Decisiones

Una decisión está lista cuando:

- existe ADR si afecta seguridad, compatibilidad, latencia u operación;
- incluye alternativas consideradas;
- describe consecuencias;
- tiene plan de validación;
- está enlazada desde documentos relevantes;
- actualiza el registro de riesgos si cambia exposición o mitigación;
- define impacto sobre seguridad, compatibilidad, latencia y operación.

## 4. Código Futuro

Una implementación estará lista cuando:

- compila;
- pasa `cargo fmt`;
- pasa `cargo clippy`;
- pasa `cargo test`;
- tiene tests de seguridad relevantes;
- no introduce `panic!` por input externo;
- no registra secretos;
- actualiza documentación afectada;
- mantiene trazabilidad entre requisito, amenaza, política, test, métrica y log;
- documenta comportamiento ante MAVLink signing si procesa paquetes firmados;
- diferencia pruebas SITL de cualquier operación con hardware real.

## 5. Seguridad

Una regla de seguridad está lista cuando:

- tiene identificador;
- tiene condición, acción y motivo;
- tiene tests positivos y negativos;
- emite log estructurado;
- incrementa métricas;
- define comportamiento ante estado desconocido.

## 6. Trazabilidad Seguridad-Test-Observabilidad

Toda regla de seguridad implementable debe tener una fila de trazabilidad:

| Campo | Descripción |
|---|---|
| Requisito | Documento y sección que justifican la regla. |
| Amenaza | ID del threat model que mitiga. |
| Regla | ID estable, por ejemplo `ARM-AUTO-001`. |
| Política | Condición, decisión y acción. |
| Tests | Unitarios, integración, SITL o fuzzing. |
| Logs | Evento y campos mínimos esperados. |
| Métricas | Contadores o histogramas afectados. |
| Riesgo residual | Qué sigue sin estar cubierto. |

Ejemplo mínimo:

| Campo | Valor |
|---|---|
| Requisito | PRD objetivo 3; `especificacion-arm-auto-001.md` |
| Amenaza | T-001 |
| Regla | `ARM-AUTO-001` |
| Política | bloquear armado no certificado en automático |
| Tests | `ARM-AUTO-U-001`, `SITL-003` |
| Logs | `security.command_blocked` |
| Métricas | `packets_blocked_total`, `commands_critical_observed_total` |
| Riesgo residual | IP allowlist no es autenticación fuerte |

## 7. MVP 0.1

MVP 0.1 está listo cuando además:

- cumple `mvp-0.1-alcance-y-criterios.md`;
- ejecuta el procedimiento de `plan-validacion-sitl.md`;
- valida `ARM-AUTO-001` en unit tests y SITL;
- no trata MAVLink signing como autenticación activa;
- limita cifrado a test harness aislado;
- deja evidencia de benchmark y fuzzing inicial.
