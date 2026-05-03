# Documentación Recomendada: Arquitectura y Producto

## 1. Objetivo

Recopilar documentación útil para definir mejor el producto y la arquitectura de MAVLink-Rust Shield Gateway.

La selección prioriza fuentes que ayuden a responder:

- Qué comportamiento real tiene MAVLink 2.0.
- Cómo enrutar y filtrar mensajes correctamente.
- Qué particularidades tienen PX4, ArduPilot y QGroundControl.
- Cómo documentar arquitectura, decisiones y riesgos.
- Cómo convertir el requerimiento inicial en producto validable.

## 2. Lecturas Prioritarias

### 2.1 MAVLink Protocol Overview

Fuente:

- https://mavlink.io/en/about/overview.html

Por qué importa:

- Es la entrada oficial al protocolo.
- Explica que MAVLink es un protocolo binario para telemetría en sistemas con restricciones de ancho de banda.
- Aclara la diferencia entre flujos de telemetría y protocolos punto a punto como misiones o parámetros.

Uso en el proyecto:

- Base para el documento de arquitectura.
- Base para explicar qué tipo de proxy estamos construyendo.
- Ayuda a separar “reenviar telemetría” de “autorizar comandos”.

### 2.2 MAVLink 2

Fuente:

- https://mavlink.io/en/guide/mavlink_2.html

Por qué importa:

- Resume las novedades de MAVLink 2: `message_id` de 24 bits, signing, extensiones, flags de compatibilidad e incompatibilidad y truncado de payload.

Uso en el proyecto:

- Justifica la decisión de usar MAVLink 2.0.
- Sirve para definir qué ocurre con paquetes MAVLink 1 si aparecen en el enlace.
- Ayuda a documentar compatibilidad hacia atrás.

### 2.3 Packet Serialization

Fuente:

- https://mavlink.io/en/guide/serialization.html

Por qué importa:

- Describe el formato real del paquete MAVLink 2 sobre el cable.
- Explica `magic`, `len`, `incompat_flags`, `compat_flags`, `seq`, `sysid`, `compid`, `msgid`, `payload`, checksum y firma.
- Es clave para entender latencia, parsing, validación y fuzzing.

Uso en el proyecto:

- Diseño del módulo `mavlink_codec`.
- Tests con paquetes corruptos.
- Fuzzing de frames.
- Decisión de no manipular bytes firmados salvo que se entienda bien el impacto.

### 2.4 MAVLink Routing

Fuente:

- https://mavlink.io/en/guide/routing.html

Por qué importa:

- Explica cómo deben enrutarse mensajes entre sistemas y componentes MAVLink.
- Detalla el uso de `target_system` y `target_component`.
- Indica que los paquetes firmados deben enrutarse sin modificar.

Uso en el proyecto:

- Diseño de la lógica de proxy.
- Reglas para no romper redes con múltiples componentes.
- Decisión pendiente: gateway transparente vs gateway que reempaqueta mensajes.

### 2.5 MAVLink Message Signing

Fuente:

- https://mavlink.io/ko/guide/message_signing.html

Por qué importa:

- MAVLink 2 ya incluye una capacidad de autenticación mediante signing.
- Esto se cruza directamente con el requerimiento de cifrado ChaCha20-Poly1305.

Uso en el proyecto:

- ADR sobre seguridad de enlace:
  - MAVLink signing.
  - Cifrado externo.
  - Gateway-to-gateway encryption.
  - Compatibilidad con GCS estándar.

Nota:

- La fuente encontrada está en ruta coreana (`/ko/`), pero el contenido describe el mecanismo oficial de signing de MAVLink 2.

### 2.6 ArduPilot: Arming and Disarming

Fuente:

- https://ardupilot.org/dev/docs/mavlink-arming-and-disarming.html

Por qué importa:

- Documenta el comando `MAV_CMD_COMPONENT_ARM_DISARM`.
- Explica `COMMAND_LONG`, `param1` para armar/desarmar y `param2=21196` para forzar armado/desarmado.

Uso en el proyecto:

- Regla de seguridad crítica `ARM-AUTO-001`.
- Tests unitarios para bloquear armado no autorizado.
- Reglas específicas para detectar force-arm.

### 2.7 ArduPilot: MAVLink Routing

Fuente:

- https://ardupilot.org/dev/docs/mavlink-routing-in-ardupilot.html

Por qué importa:

- Explica cómo ArduPilot decide procesar o reenviar mensajes según `sysid`, `compid`, `target_system` y `target_component`.

Uso en el proyecto:

- Diseño del gateway cuando haya más de un componente.
- Evitar que el proxy cambie semántica de routing.
- Casos de prueba con `target_system=0` y destinos concretos.

### 2.8 ArduPilot MAVLink Interface

Fuente:

- https://ardupilot.org/dev/docs/mavlink-commands.html

Por qué importa:

- Índice práctico de comandos MAVLink soportados por ArduPilot.

Uso en el proyecto:

- Catálogo inicial de comandos críticos.
- Priorización de reglas de sanitización.
- Futuras reglas para cambio de modo, misión, parámetros, home, RC override y guided mode.

### 2.9 PX4 MAVLink Profiles

Fuente:

- https://docs.px4.io/main/en/mavlink/mavlink_profiles

Por qué importa:

- PX4 define perfiles de streaming MAVLink como `Normal`, `Onboard`, `Config`, `Minimal`, `Low Bandwidth`, etc.

Uso en el proyecto:

- Comprender tasas de mensajes y perfiles de telemetría.
- Diseñar benchmarks realistas.
- Definir escenarios de prueba con enlaces lentos o rápidos.

### 2.10 PX4 Streaming MAVLink Messages

Fuente:

- https://docs.px4.io/main/en/mavlink/streaming_messages

Por qué importa:

- Explica cómo configurar mensajes y rates de streaming en PX4.

Uso en el proyecto:

- Definir pruebas de carga.
- Entender qué volumen de telemetría puede atravesar el gateway.
- Evaluar impacto de cifrado y logging.

### 2.11 QGroundControl MAVLink Settings

Fuente:

- https://docs.qgroundcontrol.com/Stable_V5.0/en/qgc-user-guide/settings_view/mavlink.html

Por qué importa:

- Documenta ajustes de QGC como System ID, heartbeat, protocolo MAVLink y forwarding.

Uso en el proyecto:

- Validar escenarios con QGroundControl.
- Decidir valores por defecto del gateway.
- Documentar cómo conectar QGC al proxy.

### 2.12 QGroundControl MAVLink Customisation

Fuente:

- https://docs.qgroundcontrol.com/Stable_V5.0/en/qgc-dev-guide/custom_build/mavlink.html

Por qué importa:

- Explica cómo QGC maneja dialectos MAVLink.
- QGC incluye `all.xml`, que incluye dialectos comunes.

Uso en el proyecto:

- Decidir si basta con dialecto `common`.
- Identificar riesgos si se usan dialectos ArduPilot/PX4 específicos.
- Documentar compatibilidad con GCS estándar.

## 3. Documentación de Arquitectura Recomendable

### 3.1 arc42

Fuente:

- https://arc42.org/documentation/
- https://arc42.org/overview
- https://github.com/arc42/arc42-template

Por qué importa:

- arc42 es una plantilla pragmática para documentar arquitectura de software.
- Cubre objetivos, restricciones, contexto, estrategia de solución, bloques, runtime, deployment, conceptos transversales, decisiones, calidad y riesgos.

Uso recomendado:

- Crear un documento `arquitectura-arc42-mavlink-rust-shield-gateway.md`.

Secciones especialmente útiles para este proyecto:

- Objetivos de calidad: latencia, seguridad, robustez, compatibilidad.
- Contexto y alcance: dron/simulador, GCS, operador, red, puerto serial.
- Vista de bloques: transport, codec, filter, crypto, metrics.
- Vista runtime: flujo GCS -> gateway -> dron y dron -> gateway -> GCS.
- Conceptos transversales: error handling, logging, key management, observabilidad.
- Riesgos técnicos.

### 3.2 ADRs: Architecture Decision Records

Fuentes:

- https://adr.github.io/
- https://adr.github.io/adr-templates/
- https://www.cavaro.io/templates/architecture-decision-record-adr

Por qué importa:

- El proyecto tiene decisiones que conviene no perder:
  - Rust frente a Go/Python.
  - Tokio frente a threads bloqueantes.
  - MAVLink signing frente a cifrado externo.
  - ChaCha20-Poly1305.
  - Modo conservador ante estado desconocido.
  - Compatibilidad con QGC estándar.

Uso recomendado:

- Crear carpeta `definicion/adr/`.
- Un ADR por decisión importante.

ADRs candidatos:

- `0001-usar-rust-2021.md`.
- `0002-usar-tokio-para-concurrencia.md`.
- `0003-usar-mavlink-2.md`.
- `0004-politica-ante-modo-desconocido.md`.
- `0005-cifrado-vs-compatibilidad-gcs.md`.
- `0006-soporte-udp-y-serial.md`.

### 3.3 OWASP Threat Modeling

Fuentes:

- https://cheatsheetseries.owasp.org/cheatsheets/Threat_Modeling_Cheat_Sheet.html
- https://owasp.org/www-community/Threat_Modeling

Por qué importa:

- El gateway es una pieza de seguridad y debe modelarse desde perspectiva adversarial.
- OWASP propone preguntas simples:
  - Qué estamos construyendo.
  - Qué puede salir mal.
  - Qué haremos al respecto.
  - Si hicimos un trabajo suficiente.

Uso recomendado:

- Crear `modelo-amenazas-mavlink-rust-shield-gateway.md`.

Amenazas iniciales:

- GCS no autorizada envía `ARM`.
- IP spoofing en UDP.
- Repetición de comandos.
- Paquetes corruptos causan caída del proceso.
- Clave de cifrado expuesta.
- Logging filtra datos sensibles.
- Gateway reenvía comandos a `target_system` incorrecto.
- Cifrado rompe compatibilidad con GCS y genera falsa sensación de seguridad.

## 4. Documentación de Producto Recomendable

### 4.1 PRD: Product Requirements Document

Fuentes:

- https://www.cavaro.io/templates/product-requirements-document
- https://www.prodpad.com/downloads/product-requirements-document-prd-template/
- https://www.aha.io/roadmapping/guide/templates/create/prd

Por qué importa:

- El requerimiento técnico actual explica bastante bien el “qué técnico”, pero todavía falta producto:
  - Para quién es.
  - Qué problema operativo resuelve.
  - Qué escenarios no cubre.
  - Cómo se mide el éxito.
  - Qué riesgos acepta.

Uso recomendado:

- Crear `prd-mavlink-rust-shield-gateway.md`.

Secciones sugeridas:

- Problema.
- Usuarios objetivo.
- Casos de uso.
- Objetivos.
- No objetivos.
- Requisitos funcionales.
- Requisitos no funcionales.
- Criterios de aceptación.
- Métricas de éxito.
- Riesgos.
- Release plan.

### 4.2 Opportunity Solution Tree

Fuentes:

- https://www.producttalk.org/glossary-discovery-opportunity-solution-tree/
- https://www.productplan.com/glossary/opportunity-solution-tree/
- https://www.ideaplan.io/frameworks/opportunity-solution-tree

Por qué importa:

- Evita saltar demasiado pronto a una solución técnica.
- Conecta outcome, oportunidades, soluciones y experimentos.

Uso recomendado:

- Crear un árbol simple para validar si el gateway debe priorizar:
  - Seguridad de comandos.
  - Cifrado de telemetría.
  - Compatibilidad con GCS.
  - Observabilidad.
  - Simulación/SITL.

Ejemplo inicial:

```text
Outcome:
  Reducir riesgo de comandos MAVLink no autorizados durante pruebas y operación.

Opportunities:
  Operador necesita confianza antes de conectar una GCS.
  Equipo necesita auditar comandos críticos.
  Desarrollo necesita simular ataques sin hardware real.

Solutions:
  Gateway proxy con allowlist de IPs.
  Modo audit_only.
  Filtro de comandos críticos.
  Logs de decisiones.

Experiments:
  SITL con comando ARM no autorizado.
  Benchmark de latencia.
  Prueba con QGroundControl.
```

## 5. Documentos que Conviene Crear en el Proyecto

### Prioridad Alta

- `prd-mavlink-rust-shield-gateway.md`
- `arquitectura-arc42-mavlink-rust-shield-gateway.md`
- `modelo-amenazas-mavlink-rust-shield-gateway.md`
- `adr/0001-usar-rust-2021.md`
- `adr/0002-usar-mavlink-2.md`
- `adr/0003-cifrado-vs-compatibilidad-gcs.md`

### Prioridad Media

- `catalogo-comandos-mavlink-criticos.md`
- `plan-validacion-sitl.md`
- `benchmark-latencia.md`
- `estrategia-fuzzing-mavlink.md`
- `matriz-compatibilidad-gcs-autopilotos.md`

### Prioridad Baja

- `roadmap-producto.md`
- `manual-operador.md`
- `runbook-incidentes.md`

## 6. Lectura Recomendada por Orden

Orden sugerido:

1. MAVLink Protocol Overview.
2. MAVLink 2.
3. Packet Serialization.
4. MAVLink Routing.
5. ArduPilot Arming and Disarming.
6. QGroundControl MAVLink Settings.
7. OWASP Threat Modeling.
8. arc42 Overview.
9. ADR Templates.
10. PRD Template.

Este orden permite entender primero el dominio, después la superficie de riesgo y finalmente cómo documentar decisiones y producto.

