# Arquitectura arc42: MAVLink-Rust Shield Gateway

## 1. Introducción y Objetivos

MAVLink-Rust Shield Gateway es un proxy de seguridad MAVLink 2.0 escrito en Rust. Se ubica entre una GCS y un vehículo o simulador para inspeccionar tráfico, mantener estado de vuelo conocido, aplicar políticas de autorización y cifrar flujos configurados hacia receptores compatibles.

### 1.1 Objetivos de Calidad

| Objetivo | Prioridad | Descripción |
|---|---:|---|
| Seguridad | Muy alta | Bloquear comandos críticos no autorizados y evitar fallos por input malicioso. |
| Compatibilidad | Muy alta | Mantener semántica MAVLink, routing y compatibilidad con GCS/autopilotos. |
| Robustez | Alta | Continuar funcionando ante paquetes corruptos, timeouts y errores recuperables. |
| Latencia | Alta | Mantener procesamiento interno objetivo por debajo de 1 ms por paquete. |
| Observabilidad | Alta | Explicar decisiones de seguridad y comportamiento operativo. |
| Portabilidad | Media | Ejecutar en Linux, macOS y Windows, priorizando validación Linux/SITL. |

### 1.2 Stakeholders

| Stakeholder | Interés |
|---|---|
| Desarrollador Rust | Código modular, testeable y seguro. |
| Investigador de seguridad | Simular ataques y observar decisiones. |
| Operador técnico | Evitar comandos peligrosos y obtener logs claros. |
| Arquitecto | Decisiones documentadas sobre signing, cifrado, routing y compatibilidad. |
| Usuario de GCS | Mantener flujo de telemetría y control esperado cuando la política lo permita. |

## 2. Restricciones

- Lenguaje: Rust edición 2021.
- Runtime recomendado: Tokio.
- Protocolo objetivo: MAVLink 2.0.
- Transporte operativo en MVP 0.1: UDP.
- Transporte posterior definido: Serial.
- Cifrado recomendado: ChaCha20-Poly1305 mediante RustCrypto.
- Validación inicial: SITL antes de hardware real.
- No operar drones reales por defecto.
- No romper MAVLink signing ni routing sin ADR explícito.

## 3. Contexto y Alcance

### 3.1 Contexto de Sistema

```text
              +-----------------------+
              | Ground Control Station |
              | QGC / Mission Planner  |
              +-----------+-----------+
                          |
                          | UDP MAVLink o flujo cifrado compatible
                          v
+----------------------------------------------------+
| MAVLink-Rust Shield Gateway                         |
| parsea, valida, autoriza, cifra, registra y reenvia |
+-------------------------+--------------------------+
                          |
                          | UDP o Serial MAVLink
                          v
              +-----------+-----------+
              | Simulador / Autopiloto |
              | ArduPilot / PX4 / SITL |
              +-----------------------+
```

### 3.2 Interfaces Externas

- UDP local para recibir tráfico del simulador o GCS.
- UDP remoto para reenviar a GCS o simulador.
- Puerto Serial para autopiloto real o laboratorio.
- Archivo de configuración.
- Variables de entorno para secretos.
- Salida de logs y métricas.

### 3.3 Exclusiones

- GUI.
- Base de datos.
- Gestión avanzada de usuarios.
- Operaciones autónomas sobre el vehículo.
- Alta disponibilidad.

## 4. Estrategia de Solución

La arquitectura se basa en un pipeline concurrente con separación estricta entre transporte, parsing, estado, autorización, cifrado y observabilidad.

Principios:

- Transparencia por defecto: reenviar sin modificar salvo política explícita.
- Seguridad conservadora: bloquear críticos si falta estado suficiente.
- Fallo recuperable: paquetes inválidos no derriban el proceso.
- Decisiones visibles: todo bloqueo debe ser explicable.
- Compatibilidad primero: signing, routing y GCS estándar se documentan antes de intervenir.

## 5. Vista de Bloques

### 5.1 Nivel 1

```text
cli -> config -> proxy
               -> transport
               -> mavlink_codec
               -> flight_state
               -> security_filter
               -> crypto
               -> logging/metrics
```

### 5.2 Componentes

| Componente | Responsabilidad |
|---|---|
| `cli` | Argumentos, ruta de configuración, arranque y apagado. |
| `config` | Carga, validación, defaults seguros y secretos externos. |
| `transport` | UDP, Serial, timeouts, errores y abstracción de I/O. |
| `mavlink_codec` | Parseo, validación y serialización MAVLink. |
| `flight_state` | Estado conocido: modo, armado, sysid/compid observados. |
| `security_filter` | Clasificación de mensajes, evaluación de política y decisión. |
| `crypto` | AEAD, nonce, envoltorio cifrado y validación criptográfica. |
| `proxy` | Orquestación bidireccional y concurrencia. |
| `metrics` | Contadores, histogramas y latencias. |
| `logging` | Eventos estructurados operativos y de seguridad. |

## 6. Vista Runtime

### 6.1 Flujo GCS a Vehículo

```text
GCS
 -> UDP receive
 -> mavlink_codec.parse
 -> security_filter.classify
 -> security_filter.evaluate
 -> allow/block/audit
 -> mavlink_codec.serialize
 -> transport.send_to_vehicle
```

Decisiones críticas:

- Si el mensaje no parsea: descartar, registrar y continuar.
- Si es comando crítico y falta estado: aplicar política de estado desconocido.
- Si es bloqueo: no reenviar nunca al vehículo.

### 6.2 Flujo Vehículo a GCS

```text
Vehiculo/SITL
 -> transport.receive
 -> mavlink_codec.parse
 -> flight_state.update si corresponde
 -> crypto.wrap si esta configurado
 -> transport.send_to_gcs
```

Decisiones críticas:

- El update de estado no debe bloquear el reenvío salvo error grave.
- El cifrado hacia GCS requiere receptor compatible.
- Los paquetes firmados deben tratarse de forma compatible con MAVLink signing.

## 7. Vista de Despliegue

### 7.1 SITL Local

```text
ArduPilot/PX4 SITL -> UDP localhost -> Gateway -> UDP localhost -> QGC
```

Uso principal: desarrollo, pruebas de políticas, benchmarks y fuzzing controlado.

### 7.2 Laboratorio con Serial

```text
Autopiloto USB/Serial -> Gateway en Linux -> UDP -> GCS
```

Uso futuro, fuera de MVP 0.1: laboratorio controlado. Requiere checklist de seguridad y no debe implicar operación real sin autorización explícita.

### 7.3 Gateway-to-Gateway Cifrado

```text
Vehiculo/SITL -> Gateway A -> flujo cifrado -> Gateway B -> GCS
```

Uso posible: preservar compatibilidad con GCS estándar descifrando antes de la GCS.

## 8. Conceptos Transversales

### 8.1 Errores

- Errores recuperables: paquete inválido, timeout, origen no permitido, fallo de descifrado.
- Errores fatales: configuración inválida, clave inválida, puerto imposible de abrir al arrancar si es requerido.
- El proceso no debe hacer `panic!` por input externo.

### 8.2 Configuración

La configuración debe separar:

- transporte;
- política de seguridad;
- criptografía;
- logging;
- métricas;
- perfil de compatibilidad.

### 8.3 Observabilidad

Eventos mínimos:

- startup y configuración efectiva no sensible;
- apertura de transportes;
- cambio de modo detectado;
- comando crítico observado;
- comando bloqueado;
- error de parseo;
- error criptográfico;
- shutdown.

### 8.4 Seguridad

- Input no confiable.
- Deny-by-default para críticos en estado desconocido.
- No logging de secretos.
- Validación estricta de configuración.
- Separación entre identidad débil por IP y autenticación fuerte futura.

### 8.5 Criptografía

ChaCha20-Poly1305 protege confidencialidad e integridad del flujo configurado, pero no preserva compatibilidad con una GCS que espere MAVLink plano. El diseño debe definir:

- formato de envoltorio;
- origen del nonce;
- gestión de clave;
- comportamiento ante fallo de autenticación;
- límites de despliegue.

## 9. Decisiones de Arquitectura Pendientes o Iniciales

- Usar Rust 2021.
- Usar Tokio.
- Usar MAVLink 2.0 como objetivo principal.
- Gateway transparente por defecto.
- Bloquear comandos críticos ante modo desconocido.
- Cifrado externo solo como test harness aislado en MVP 0.1.
- ArduPilot SITL y QGroundControl como objetivos iniciales.
- Definir comportamiento exacto ante MAVLink signing.

Estas decisiones deben quedar registradas como ADRs.

## 10. Riesgos Técnicos

| Riesgo | Impacto | Mitigación |
|---|---:|---|
| Cifrado rompe GCS estándar | Alto | ADR y matriz de compatibilidad. |
| Modo automático mal inferido | Alto | Tests por autopiloto y dialecto. |
| Routing incorrecto | Alto | Preservar mensajes y probar target fields. |
| Signing roto por modificación | Alto | Modo transparente para firmados o ADR. |
| Latencia > 1 ms | Medio | Benchmarks por etapa. |
| Parser vulnerable | Alto | Fuzzing y tests negativos. |
| Logs sensibles | Medio | Especificación de observabilidad. |

## 11. Calidad y Validación

- Unit tests para config, clasificación y política.
- Integration tests para proxy UDP.
- SITL tests con GCS real.
- Fuzzing de parser/filtro/wrapper.
- Benchmarks con y sin cifrado.
- Revisión de logs para ausencia de secretos.
- Pruebas de compatibilidad con QGroundControl.

## 12. Glosario Corto

- GCS: Ground Control Station.
- SITL: Software-in-the-loop simulator.
- AEAD: cifrado autenticado con datos asociados.
- Signing: autenticación nativa de mensajes MAVLink 2.
- Wrapper: formato externo que encapsula MAVLink, por ejemplo para cifrado.
- Modo transparente: reenvío sin modificar bytes semánticos del mensaje.
