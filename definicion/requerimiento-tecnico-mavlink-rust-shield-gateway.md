# Requerimiento Técnico: MAVLink-Rust Shield Gateway

## 1. Resumen Ejecutivo

MAVLink-Rust Shield Gateway es una aplicación de consola desarrollada en Rust que actúa como pasarela de seguridad entre un dron, o simulador compatible con MAVLink, y una Estación de Control de Tierra (GCS). Su función principal es interceptar el tráfico MAVLink, validar los mensajes, aplicar reglas de autorización y cifrar el flujo de datos saliente hacia la GCS.

El sistema se plantea como una primera capa defensiva para escenarios de operación, pruebas y simulación, reduciendo el riesgo de que comandos no autorizados lleguen al vehículo, especialmente durante modos de vuelo sensibles como el modo automático.

## 2. Objetivo del Proyecto

Desarrollar una aplicación de consola en Rust que funcione como proxy bidireccional entre:

- Un dron real conectado por puerto serie o un simulador expuesto por UDP.
- Una GCS conectada por UDP.

La pasarela debe procesar paquetes MAVLink 2.0 en tiempo real, inspeccionar los mensajes relevantes, bloquear comandos no autorizados y cifrar la telemetría o el flujo saliente hacia la estación de tierra cuando esté configurado.

## 3. Alcance Inicial

### Incluido

- Ejecución como aplicación CLI.
- Soporte para transporte UDP.
- Soporte para transporte Serial.
- Parser de MAVLink 2.0.
- Proxy bidireccional dron/simulador <-> gateway <-> GCS.
- Filtro de seguridad basado en `message_id`, modo de vuelo y origen del comando.
- Lista configurable de IPs certificadas.
- Cifrado autenticado del canal de salida hacia la GCS.
- Configuración externa mediante archivo.
- Logging operativo básico.
- Métricas mínimas de latencia, mensajes procesados, mensajes bloqueados y errores.

### Fuera de Alcance en la Primera Versión

- Interfaz gráfica.
- Gestión avanzada de usuarios.
- Rotación automática de claves criptográficas.
- Detección completa de intrusiones.
- Soporte completo para todos los dialectos MAVLink personalizados.
- Persistencia en base de datos.
- Alta disponibilidad o clustering.

## 4. Contexto Operativo

El gateway se ejecutará en una máquina intermedia, por ejemplo un portátil de operaciones, una Raspberry Pi o un equipo de pruebas. En escenarios de simulación, recibirá paquetes MAVLink desde herramientas como SITL, PX4, ArduPilot o Gazebo. En escenarios reales, recibirá tráfico desde un autopiloto conectado por USB/Serial.

La GCS no se conectará directamente al dron, sino al gateway. Esto permite inspeccionar y controlar el tráfico antes de reenviarlo.

Ejemplo de topología:

```text
Simulador o Dron <----> MAVLink-Rust Shield Gateway <----> GCS
        UDP/Serial                  Rust CLI                  UDP
```

## 5. Especificaciones Funcionales

### 5.1 Conectividad Dual

El sistema debe permitir seleccionar el origen del tráfico MAVLink mediante configuración:

- `udp`: para simuladores o drones que expongan telemetría por red.
- `serial`: para drones reales conectados por USB, UART o adaptador serie.

Para UDP, la aplicación debe permitir configurar:

- Dirección local de escucha.
- Dirección remota del dron o simulador.
- Dirección remota de la GCS.
- Timeout de lectura.
- Tamaño máximo de buffer.

Para Serial, la aplicación debe permitir configurar:

- Ruta del puerto, por ejemplo `/dev/ttyACM0`, `/dev/ttyUSB0` o `COM3`.
- Baud rate.
- Bits de datos.
- Paridad.
- Bits de parada.
- Timeout de lectura.

### 5.2 Protocolo MAVLink 2.0

El sistema debe implementar el parseo de paquetes MAVLink 2.0 utilizando un crate especializado. Debe poder:

- Leer frames MAVLink desde UDP o Serial.
- Validar el formato básico del frame.
- Obtener `message_id`.
- Obtener `system_id` y `component_id`.
- Diferenciar mensajes de telemetría y comandos.
- Reenviar mensajes válidos preservando su contenido, salvo que se aplique una transformación explícita.
- Registrar errores de parseo sin detener el proceso principal.

El dialecto MAVLink inicial recomendado será `common`, salvo que el proyecto requiera integrar dialectos específicos de PX4, ArduPilot u otro autopiloto.

### 5.3 Proxy Bidireccional

El gateway debe manejar dos flujos principales:

- Flujo ascendente: dron/simulador -> gateway -> GCS.
- Flujo descendente: GCS -> gateway -> dron/simulador.

Ambos flujos deben ejecutarse de forma concurrente para evitar bloqueos. La arquitectura debe impedir que una operación lenta en un canal degrade el otro canal.

### 5.4 Filtro de Seguridad

El filtro de seguridad debe inspeccionar cada mensaje recibido desde la GCS antes de reenviarlo al dron.

Regla inicial obligatoria:

- Si el dron está en modo `Automático`, el gateway debe bloquear cualquier comando de armado que no provenga de una IP certificada.

La regla debe considerar:

- `message_id` del paquete MAVLink.
- Tipo de comando, especialmente comandos relacionados con armado/desarmado.
- Modo de vuelo conocido más reciente.
- Dirección IP de origen cuando el transporte sea UDP.
- Configuración de IPs certificadas.

Mensajes MAVLink relevantes para la primera versión:

- `COMMAND_LONG`.
- `COMMAND_INT`.
- `HEARTBEAT`.

Comandos relevantes:

- `MAV_CMD_COMPONENT_ARM_DISARM`.

El estado de modo automático debe inferirse inicialmente desde mensajes `HEARTBEAT`, revisando los campos de modo disponibles en el dialecto MAVLink usado.

Cuando un mensaje sea bloqueado, el sistema debe:

- No reenviar el paquete al dron.
- Registrar el evento con timestamp, IP de origen, `message_id`, motivo de bloqueo y estado de vuelo conocido.
- Incrementar una métrica de comandos bloqueados.
- Opcionalmente notificar a la GCS mediante un mensaje de estado, si se define como comportamiento deseado.

### 5.5 Cifrado en Tiempo Real

El gateway debe implementar una capa de cifrado autenticado para el flujo saliente hacia la GCS.

Algoritmo recomendado:

- `ChaCha20-Poly1305`.

Motivos:

- Buen rendimiento en hardware sin aceleración AES.
- Cifrado autenticado AEAD.
- Adecuado para flujos de baja latencia.
- Disponible en crates mantenidos del ecosistema RustCrypto.

El cifrado debe proteger:

- Confidencialidad del payload enviado a la GCS.
- Integridad del mensaje cifrado.
- Detección de manipulación mediante tag de autenticación.

Consideraciones iniciales:

- La clave debe cargarse desde configuración, variable de entorno o fichero seguro.
- El nonce no debe reutilizarse con la misma clave.
- Debe definirse un formato de envoltorio para transportar nonce, ciphertext y tag.
- La GCS o un cliente receptor compatible debe poder descifrar el flujo.

Nota de definición: si se necesita compatibilidad directa con una GCS estándar sin modificar, el cifrado no podrá aplicarse directamente al flujo MAVLink esperado por esa GCS. En ese caso se requiere un adaptador receptor, un plugin de GCS o limitar el cifrado a un enlace gateway-gateway.

## 6. Especificaciones Técnicas

### 6.1 Lenguaje

- Rust.
- Edición 2021.

### 6.2 Runtime y Concurrencia

- `tokio` para tareas asíncronas.
- Canales internos para desacoplar lectura, validación, cifrado y escritura.
- Manejo de cancelación ordenada con `Ctrl+C`.

### 6.3 Crates Recomendados

Crates base:

- `mavlink`: parseo y serialización de mensajes MAVLink.
- `tokio`: runtime asíncrono, UDP y tareas concurrentes.
- `tokio-serial`: integración serial asíncrona con Tokio.
- `serde`: serialización y deserialización.
- `toml` o `serde_yaml`: configuración externa.
- `tracing`: logging estructurado.
- `tracing-subscriber`: configuración de salida de logs.
- `clap`: argumentos de línea de comandos.

Crates criptográficos recomendados:

- `chacha20poly1305`: implementación AEAD de ChaCha20-Poly1305.
- `rand` o `getrandom`: generación de nonces cuando aplique.
- `zeroize`: limpieza de secretos en memoria cuando sea razonable.

Nota: se recomienda preferir crates actuales del ecosistema RustCrypto frente a `rust-crypto`, ya que algunas librerías históricas pueden estar obsoletas o menos mantenidas.

### 6.4 Perfil de Rendimiento

Objetivo de latencia:

- Latencia máxima de procesamiento interno inferior a `1 ms` por paquete en condiciones normales.

La medición debe excluir latencias externas de red, driver serial, simulador, autopiloto y GCS.

Métricas mínimas:

- Tiempo de parseo.
- Tiempo de evaluación de reglas.
- Tiempo de cifrado.
- Tiempo total dentro del gateway.
- Mensajes por segundo.
- Mensajes descartados por error.
- Mensajes bloqueados por política.

### 6.5 Configuración

La aplicación debe aceptar una ruta de configuración:

```bash
mavlink-shield-gateway --config config.toml
```

Ejemplo orientativo de configuración:

```toml
[transport]
mode = "udp"

[udp]
listen_drone = "0.0.0.0:14550"
gcs_addr = "127.0.0.1:14551"
drone_addr = "127.0.0.1:14540"

[serial]
port = "/dev/ttyACM0"
baud_rate = 57600

[security]
certified_ips = ["127.0.0.1", "192.168.1.50"]
block_arm_in_auto_mode = true

[crypto]
enabled = true
algorithm = "chacha20poly1305"
key_source = "env"
key_env = "MAVLINK_SHIELD_KEY"

[logging]
level = "info"
```

## 7. Arquitectura Propuesta

### 7.1 Componentes

Componentes iniciales:

- `cli`: lectura de argumentos y arranque.
- `config`: carga y validación de configuración.
- `transport`: abstracción para UDP y Serial.
- `mavlink_codec`: parseo y serialización de frames MAVLink.
- `security_filter`: reglas de autorización y sanitización.
- `flight_state`: estado conocido del dron, incluyendo modo de vuelo.
- `crypto`: cifrado y descifrado cuando aplique.
- `proxy`: orquestación de flujos concurrentes.
- `metrics`: contadores y mediciones de latencia.
- `logging`: logs estructurados.

### 7.2 Flujo de Comandos GCS -> Dron

```text
GCS
 -> UDP receive
 -> parse MAVLink
 -> inspect message_id
 -> evaluate security policy
 -> allow or block
 -> serialize MAVLink
 -> send to drone/simulator
```

### 7.3 Flujo de Telemetría Dron -> GCS

```text
Dron/Simulador
 -> UDP/Serial receive
 -> parse MAVLink
 -> update flight state when needed
 -> serialize or wrap
 -> encrypt when enabled
 -> send to GCS endpoint
```

## 8. Reglas de Seguridad Iniciales

### 8.1 Regla ARM-AUTO-001

Nombre:

- Bloqueo de armado no certificado en modo automático.

Descripción:

- Si el estado actual del dron indica modo automático y llega un comando de armado desde una IP no certificada, el comando debe ser bloqueado.

Entrada:

- Mensaje MAVLink desde la GCS.
- IP de origen.
- Estado de vuelo conocido.

Condición:

- `flight_mode == automatic`.
- `command == MAV_CMD_COMPONENT_ARM_DISARM`.
- `source_ip` no pertenece a `certified_ips`.

Acción:

- Bloquear mensaje.
- Registrar evento.
- Incrementar contador.

Resultado esperado:

- El dron no recibe el comando.

### 8.2 Comportamiento ante Estado Desconocido

Debe definirse una política para el caso en que el modo de vuelo todavía no sea conocido. Recomendación inicial:

- Política conservadora: bloquear comandos críticos hasta recibir un `HEARTBEAT` válido.

Esta política puede ser configurable:

- `allow`.
- `block`.
- `audit_only`.

## 9. Requisitos No Funcionales

### 9.1 Seguridad

- Validar todo input externo.
- No asumir que los paquetes MAVLink son correctos.
- Evitar `panic!` ante tráfico malformado.
- No registrar claves ni material sensible.
- Separar configuración sensible de configuración normal cuando sea posible.
- Usar cifrado autenticado, no solo cifrado simétrico sin autenticación.

### 9.2 Fiabilidad

- La aplicación debe seguir ejecutándose ante paquetes corruptos.
- Los errores recuperables deben registrarse y contabilizarse.
- Debe existir apagado ordenado con cierre de sockets y puerto serial.

### 9.3 Observabilidad

Logs mínimos:

- Inicio y configuración efectiva no sensible.
- Apertura de sockets o puerto serial.
- Cambios de modo detectados.
- Comandos bloqueados.
- Errores de parseo.
- Errores de cifrado.
- Cierre de aplicación.

Métricas mínimas:

- `packets_received_total`.
- `packets_forwarded_total`.
- `packets_blocked_total`.
- `packets_parse_error_total`.
- `processing_latency_ms`.

### 9.4 Portabilidad

El sistema debe poder desarrollarse y ejecutarse en:

- Linux.
- macOS.
- Windows.

La operación con drones reales se validará primero en Linux por disponibilidad habitual de dispositivos `/dev/tty*`.

## 10. Entorno de Desarrollo en VS Code

Extensiones recomendadas:

- `rust-analyzer`: autocompletado, navegación, diagnóstico de tipos y soporte del lenguaje.
- `CodeLLDB`: depuración nativa de binarios Rust.
- `crates`: gestión visual de versiones en `Cargo.toml`.
- `Even Better TOML`: edición cómoda de archivos `.toml`.

Configuración recomendada:

- Formateo con `rustfmt`.
- Linting con `clippy`.
- Tests con `cargo test`.
- Depuración con breakpoints en parser, filtro de seguridad y capa de transporte.

Comandos útiles:

```bash
cargo fmt
cargo clippy
cargo test
cargo run -- --config config.toml
```

## 11. Estrategia de Pruebas

### 11.1 Pruebas Unitarias

Casos mínimos:

- Parseo de configuración válida.
- Rechazo de configuración inválida.
- Detección de comando de armado.
- Detección de modo automático desde `HEARTBEAT`.
- Bloqueo de comando de armado desde IP no certificada.
- Permitir comando de armado desde IP certificada.
- Comportamiento con modo desconocido.
- Cifrado y descifrado correcto.
- Fallo de descifrado ante mensaje manipulado.

### 11.2 Pruebas de Integración

Casos mínimos:

- Flujo UDP simulador -> gateway -> GCS.
- Flujo UDP GCS -> gateway -> simulador.
- Bloqueo de comando no autorizado.
- Reenvío de comando autorizado.
- Cierre ordenado.

### 11.3 Pruebas de Rendimiento

Objetivo:

- Verificar que el procesamiento interno promedio y percentil alto se mantienen por debajo del objetivo definido.

Métricas recomendadas:

- Latencia p50.
- Latencia p95.
- Latencia p99.
- Paquetes por segundo.
- Uso de CPU.
- Uso de memoria.

## 12. Criterios de Aceptación Iniciales

El requerimiento se considerará satisfecho en una primera versión cuando:

- La aplicación arranque desde consola con archivo de configuración.
- Pueda abrir un transporte UDP.
- Pueda abrir un transporte Serial.
- Pueda parsear mensajes MAVLink 2.0.
- Pueda reenviar telemetría entre dron/simulador y GCS.
- Pueda identificar `HEARTBEAT`, `COMMAND_LONG` y `COMMAND_INT`.
- Pueda mantener estado básico del modo de vuelo.
- Bloquee comandos de armado no autorizados en modo automático.
- Permita comandos autorizados desde IPs certificadas.
- Cifre el flujo saliente hacia la GCS cuando esté habilitado.
- Registre eventos de seguridad.
- Incluya pruebas automatizadas de las reglas principales.
- Documente cómo ejecutar una prueba con simulador.

## 13. Riesgos y Decisiones Pendientes

### 13.1 Compatibilidad con GCS Estándar

Riesgo:

- Si se cifra directamente el tráfico hacia la GCS, una GCS estándar no podrá interpretar los paquetes MAVLink.

Decisión pendiente:

- Definir si existirá un cliente receptor compatible, un segundo gateway de descifrado o un plugin específico para la GCS.

### 13.2 Interpretación de Modo Automático

Riesgo:

- La detección de modo automático puede variar entre autopilotos y dialectos.

Decisión pendiente:

- Confirmar autopiloto objetivo inicial: ArduPilot, PX4 u otro.

### 13.3 Latencia Inferior a 1 ms

Riesgo:

- El objetivo es exigente si se incluye cifrado, serial lento, logging síncrono o hardware limitado.

Decisión pendiente:

- Definir hardware objetivo y metodología de medición.

### 13.4 Gestión de Claves

Riesgo:

- Una clave estática mal gestionada reduce significativamente la seguridad real.

Decisión pendiente:

- Definir mecanismo inicial de provisión, almacenamiento y rotación de claves.

## 14. Roadmap Propuesto

### Fase 0: Definición

- Refinar alcance.
- Elegir autopiloto objetivo.
- Elegir modo de compatibilidad con GCS.
- Definir formato de configuración.
- Definir política ante estado desconocido.

### Fase 1: Prototipo UDP

- CLI básica.
- Configuración TOML.
- Transporte UDP.
- Parseo MAVLink.
- Reenvío bidireccional.
- Logging básico.

### Fase 2: Filtro de Seguridad

- Estado de vuelo desde `HEARTBEAT`.
- Detección de comandos críticos.
- Lista de IPs certificadas.
- Bloqueo de armado no autorizado.
- Tests unitarios de reglas.

### Fase 3: Serial y Dron Real

- Transporte Serial.
- Pruebas con autopiloto real o hardware de laboratorio.
- Manejo robusto de desconexiones.

### Fase 4: Cifrado

- Implementar ChaCha20-Poly1305.
- Definir envoltorio de mensajes cifrados.
- Pruebas de integridad.
- Medición de impacto en latencia.

### Fase 5: Hardening

- Métricas.
- Benchmarks.
- Documentación de operación.
- Pruebas de estrés.
- Revisión de seguridad.

## 15. Preguntas Abiertas

- ¿El objetivo inicial es ArduPilot, PX4 o ambos?
- ¿La GCS será QGroundControl, Mission Planner u otra?
- ¿Se aceptará modificar la GCS o debe mantenerse compatibilidad nativa?
- ¿El cifrado debe aplicarse solo a telemetría, solo a comandos o a ambos flujos?
- ¿Qué IPs se considerarán certificadas y cómo se administrarán?
- ¿La política ante modo desconocido debe ser bloquear, permitir o solo auditar?
- ¿El sistema debe emitir alertas visibles para el operador?
- ¿El gateway debe poder funcionar en modo `audit_only` sin bloquear?
- ¿Qué hardware se usará como plataforma mínima?

