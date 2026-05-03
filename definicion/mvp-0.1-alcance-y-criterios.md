# MVP 0.1: Alcance y Criterios

## 1. Objetivo

Cerrar el alcance de la primera versión implementable para evitar que el proyecto intente resolver simultáneamente UDP, Serial, cifrado operativo, múltiples autopilotos, signing completo y catálogo amplio de comandos.

MVP 0.1 debe demostrar que el gateway puede operar como proxy MAVLink 2.0 seguro en simulación, con una regla crítica testeable, observabilidad suficiente y compatibilidad básica con una GCS estándar.

## 2. Alcance Cerrado

MVP 0.1 incluye:

- transporte UDP;
- entorno SITL;
- QGroundControl como GCS primaria;
- ArduPilot SITL como autopiloto primario;
- MAVLink 2.0 como protocolo objetivo;
- proxy bidireccional GCS <-> gateway <-> SITL;
- parsing de `HEARTBEAT`, `COMMAND_LONG` y `COMMAND_INT`;
- estado mínimo de vuelo desde `HEARTBEAT`;
- política `ARM-AUTO-001`;
- política `CRITICAL-UNKNOWN-001`;
- logs estructurados de decisiones;
- métricas mínimas;
- benchmark básico de latencia interna;
- fuzzing inicial de parser/filtro/configuración;
- cifrado ChaCha20-Poly1305 solo como test harness aislado, no como flujo compatible con QGroundControl.

## 3. Fuera de Alcance en MVP 0.1

- Serial con autopiloto real.
- Operación con hardware real.
- PX4 como autopiloto primario.
- Mission Planner como GCS primaria.
- MAVLink signing completo.
- Reescritura o reserialización intencional de paquetes firmados.
- Cifrado directo hacia QGroundControl estándar.
- Gestión avanzada de claves.
- Rotación automática de claves.
- Bloqueo completo de misiones, parámetros, RC override o guided mode.
- GUI.

## 4. Decisiones de Alcance

### 4.1 Autopiloto Primario

ArduPilot SITL será el autopiloto primario para MVP 0.1.

Motivo:

- documentación clara de armado/desarmado por MAVLink;
- disponibilidad habitual en entornos SITL;
- buena compatibilidad con QGroundControl y MAVProxy.

### 4.2 GCS Primaria

QGroundControl será la GCS primaria para MVP 0.1.

Motivo:

- herramienta estándar y ampliamente usada;
- documentación pública de configuración MAVLink;
- permite validar compatibilidad con una GCS real sin desarrollar UI propia.

### 4.3 Cifrado

El cifrado no entra como flujo operativo compatible con QGroundControl en MVP 0.1.

Sí entra como:

- módulo diseñado;
- pruebas unitarias;
- test harness de cifrado/descifrado;
- validación de gestión de nonce y errores;
- documentación de incompatibilidad con GCS estándar.

### 4.4 MAVLink Signing

MVP 0.1 no implementa validación criptográfica completa de MAVLink signing.

Comportamiento de MVP 0.1:

- detectar paquetes MAVLink 2 firmados cuando el parser/crate exponga esa información o cuando pueda observarse el flag correspondiente;
- preservar transparencia;
- no modificar mensajes firmados;
- no afirmar autenticidad si no se valida la firma;
- registrar limitación.

## 5. Criterios de Aceptación

MVP 0.1 está aceptado cuando:

- QGroundControl recibe telemetría de ArduPilot SITL a través del gateway.
- ArduPilot SITL recibe comandos permitidos a través del gateway.
- El gateway observa `HEARTBEAT` y mantiene estado mínimo.
- `ARM-AUTO-001` tiene tests unitarios y prueba SITL documentada.
- `CRITICAL-UNKNOWN-001` bloquea comandos críticos antes de estado válido.
- Paquetes malformados no provocan `panic!`.
- Logs de bloqueo contienen regla, decisión, origen, mensaje y motivo.
- Logs no contienen claves ni payload completo por defecto.
- Métricas mínimas se incrementan correctamente.
- Benchmark nominal reporta p50/p95/p99 de latencia interna.
- Fuzzing inicial existe para parser/filtro/configuración.
- Cifrado tiene test harness, pero no se presenta como compatible con QGroundControl estándar.

## 6. Trazabilidad

| Tema | Documento relacionado |
|---|---|
| Producto | `prd-mavlink-rust-shield-gateway.md` |
| Amenazas | `modelo-amenazas-mavlink-rust-shield-gateway.md` |
| Arquitectura | `arquitectura-arc42-mavlink-rust-shield-gateway.md` |
| Signing | `adr/0008-comportamiento-ante-mavlink-signing.md` |
| ARM-AUTO-001 | `especificacion-arm-auto-001.md` |
| SITL | `plan-validacion-sitl.md` |
| Cifrado | `adr/0004-cifrado-vs-compatibilidad-gcs.md` |

