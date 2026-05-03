# PRD: MAVLink-Rust Shield Gateway

## 1. Estado del Documento

- Estado: borrador inicial endurecido para MVP 0.1.
- Fase: definición de producto, arquitectura y validación SITL.
- Alcance operativo permitido por defecto: simulación, SITL, documentación, pruebas y análisis.
- Alcance operativo excluido por defecto: control de drones reales, armado, desarmado, cambio de modo, carga de misión, override RC o mutación de parámetros.

## 2. Resumen

MAVLink-Rust Shield Gateway es una aplicación de consola en Rust que actúa como pasarela de seguridad entre un vehículo MAVLink, real o simulado, y una Ground Control Station (GCS). Su propósito es reducir el riesgo de que comandos no autorizados, malformados, repetidos o incompatibles alcancen el autopiloto, sin degradar de forma significativa la telemetría ni la experiencia de operación en entornos de prueba.

El producto se diseña primero para entornos SITL y laboratorio. La operación con hardware real queda fuera de la primera validación funcional y requerirá procedimientos específicos, autorización humana explícita y controles adicionales.

## 3. Problema

MAVLink es ampliamente utilizado para telemetría y control de vehículos no tripulados. En muchos entornos de desarrollo y pruebas, una GCS se conecta directamente a un simulador o autopiloto por UDP o Serial. Esa conexión directa deja poca capacidad intermedia para:

- auditar comandos críticos antes de que lleguen al vehículo;
- aplicar políticas locales de autorización;
- observar intentos de control no deseados;
- simular escenarios adversariales de forma controlada;
- medir impacto de seguridad, cifrado y filtrado sobre latencia;
- estudiar compatibilidad real con GCS y autopilotos.

La necesidad inicial no es sustituir al autopiloto ni convertir el gateway en una GCS. La necesidad es introducir un punto explícito de inspección, autorización, trazabilidad y experimentación segura.

## 4. Usuarios y Actores

### 4.1 Usuarios Objetivo

- Desarrollador de sistemas UAV que necesita validar reglas de seguridad MAVLink en SITL.
- Investigador de seguridad que necesita reproducir comandos críticos y tráfico malformado sin tocar hardware real.
- Operador técnico que necesita visibilidad sobre comandos enviados desde una GCS.
- Equipo de arquitectura que necesita una base documentada para decidir compatibilidad, cifrado, signing y políticas.

### 4.2 Actores Técnicos

- GCS: QGroundControl, Mission Planner u otra estación compatible.
- Gateway: proceso Rust que recibe, valida, autoriza, registra, cifra cuando aplique y reenvía.
- Vehículo o simulador: ArduPilot SITL, PX4 SITL, Gazebo u otro sistema que emite y recibe MAVLink.
- Operador humano: responsable de arrancar, configurar y observar el gateway.
- Configuración externa: archivo y variables de entorno que definen transportes, políticas y secretos.

## 5. Objetivos

1. Construir un proxy MAVLink 2.0 bidireccional para UDP en MVP 0.1, dejando Serial para una fase posterior.
2. Separar claramente telemetría, comandos y estado de vuelo conocido.
3. Aplicar una primera regla crítica: bloquear armado no certificado cuando el vehículo esté en modo automático.
4. Registrar eventos operativos y de seguridad con suficiente contexto para auditoría.
5. Mantener latencia interna objetivo inferior a 1 ms por paquete en condiciones normales de referencia.
6. Permitir cifrado autenticado ChaCha20-Poly1305 en flujos configurados, dejando explícitas sus implicaciones de compatibilidad.
7. Validar primero con ArduPilot SITL y QGroundControl antes de considerar hardware real.

## 6. No Objetivos

- No reemplazar una GCS.
- No implementar una interfaz gráfica.
- No garantizar seguridad completa de MAVLink por sí solo.
- No considerar la IP UDP como identidad fuerte.
- No operar drones reales en la fase inicial.
- No implementar alta disponibilidad, clustering ni persistencia en base de datos.
- No soportar desde el inicio todos los dialectos MAVLink personalizados.
- No mutar parámetros, misiones o modos salvo que una política futura lo autorice explícitamente en simulación.

## 7. Escenarios Principales

### 7.1 SITL con GCS Estándar

Un desarrollador ejecuta ArduPilot SITL o PX4 SITL. La GCS se conecta al gateway en lugar de conectarse directamente al simulador. El gateway reenvía telemetría y comandos permitidos, bloquea comandos críticos no autorizados y registra las decisiones.

### 7.2 Detección de Armado No Autorizado

El vehículo está en modo automático según el último `HEARTBEAT` válido. Una GCS no certificada envía `MAV_CMD_COMPONENT_ARM_DISARM` mediante `COMMAND_LONG` o `COMMAND_INT`. El gateway bloquea el mensaje, no lo reenvía al vehículo, registra el evento e incrementa métricas.

### 7.3 Auditoría sin Bloqueo

En un entorno de investigación, la política puede ejecutarse en modo `audit_only` para medir qué se habría bloqueado sin afectar al flujo. Este modo nunca debe ser el valor seguro por defecto para comandos críticos.

### 7.4 Cifrado Gateway a Receptor Compatible

El gateway cifra el flujo saliente hacia un receptor que conoce el formato de envoltorio y la clave. Si la GCS es estándar y no puede descifrar, el modo cifrado directo no es compatible. Esta restricción debe ser visible en configuración y documentación.

## 8. Requisitos Funcionales

### 8.1 CLI y Configuración

- La aplicación debe arrancar desde consola.
- Debe aceptar ruta de configuración mediante `--config`.
- Debe validar configuración antes de abrir transportes.
- Debe rechazar configuraciones ambiguas, inseguras o incompletas.
- Debe permitir perfiles de simulación y laboratorio.

### 8.2 Transporte

- Debe soportar UDP para SITL y GCS en MVP 0.1.
- Debe dejar Serial definido para escenarios posteriores con autopiloto conectado por USB/UART.
- Debe abstraer lectura y escritura para no acoplar el filtro a un transporte concreto.
- Debe manejar timeouts y errores recuperables sin terminar el proceso principal.

### 8.3 MAVLink

- Debe parsear MAVLink 2.0 con un crate especializado.
- Debe extraer `message_id`, `system_id`, `component_id` y campos de destino cuando existan.
- Debe procesar inicialmente `HEARTBEAT`, `COMMAND_LONG` y `COMMAND_INT`.
- Debe preservar el contenido de los mensajes reenviados salvo decisión documentada.
- Debe registrar errores de parseo sin `panic!`.

### 8.4 Estado de Vuelo

- Debe mantener el último estado de vuelo conocido.
- Debe inferir modo automático inicialmente desde `HEARTBEAT`.
- Debe registrar cambios de modo observados.
- Debe definir política explícita para modo desconocido.

### 8.5 Filtro de Seguridad

- Debe inspeccionar comandos provenientes de la GCS antes de reenviarlos.
- Debe detectar `MAV_CMD_COMPONENT_ARM_DISARM`.
- Debe evaluar IP de origen cuando el transporte sea UDP.
- Debe aplicar allowlist de IPs certificadas como control inicial, no como autenticación fuerte.
- Debe soportar al menos acciones `allow`, `block` y `audit_only`.

### 8.6 Cifrado

- Debe soportar ChaCha20-Poly1305 como cifrado autenticado.
- Debe impedir reutilización de nonce con la misma clave.
- Debe separar material secreto de logs y errores.
- Debe documentar el formato de envoltorio cifrado.
- Debe fallar de forma segura si la configuración criptográfica es inválida.

### 8.7 Observabilidad

- Debe usar logs estructurados.
- Debe emitir métricas de mensajes recibidos, reenviados, bloqueados, descartados y errores.
- Debe medir latencia interna por etapas relevantes.
- Debe permitir correlacionar eventos sin exponer secretos.

## 9. Requisitos No Funcionales

### 9.1 Seguridad

- Todo input MAVLink se trata como no confiable.
- El gateway no debe caerse por tráfico malformado.
- Las decisiones de bloqueo deben ser trazables.
- La configuración debe favorecer valores seguros.
- Las claves no deben imprimirse ni persistirse accidentalmente.

### 9.2 Compatibilidad

- El diseño debe priorizar MAVLink 2.0.
- Debe documentar comportamiento ante MAVLink 1.
- Debe no romper routing ni signing sin una decisión explícita.
- Debe validar escenarios con QGroundControl y al menos un autopiloto SITL.

### 9.3 Rendimiento

- Objetivo: latencia interna inferior a 1 ms por paquete en escenario nominal.
- Deben medirse p50, p95 y p99.
- Debe diferenciarse latencia interna de latencia de red, driver serial, GCS o simulador.

### 9.4 Portabilidad

- Desarrollo objetivo: Linux, macOS y Windows.
- Validación prioritaria: Linux con SITL.
- Serial real se considerará después de validación simulada.

## 10. Métricas de Éxito

- El gateway reenvía tráfico SITL entre simulador y GCS.
- Bloquea el armado no autorizado en modo automático.
- No bloquea comandos permitidos por política.
- No registra secretos.
- Mantiene latencia interna bajo el objetivo en benchmark nominal.
- Genera logs suficientes para explicar cada decisión de seguridad.
- Los documentos de amenazas, arquitectura y ADRs cubren las decisiones principales.

## 11. Criterios de Aceptación de la Primera Versión

- CLI con configuración externa validada.
- Transporte UDP funcional.
- Transporte Serial documentado como fuera de alcance operativo en MVP 0.1.
- Parser MAVLink 2.0 integrado.
- Proxy bidireccional concurrente.
- Estado básico desde `HEARTBEAT`.
- Regla `ARM-AUTO-001` implementada y testeada.
- Métricas y logs mínimos.
- Modo cifrado definido y probado con test harness aislado; no compatible directamente con QGroundControl estándar en MVP 0.1.
- Plan SITL ejecutable documentado.

## 12. Riesgos Principales

- Cifrado directo incompatible con GCS estándar.
- Interpretación de modo diferente entre ArduPilot y PX4.
- UDP source IP insuficiente como identidad.
- MAVLink signing puede romperse si el gateway reserializa o modifica mensajes.
- Latencia objetivo puede ser irreal en hardware limitado con logging o cifrado intensivo.
- Comandos críticos adicionales pueden quedar fuera del filtro inicial.

## 13. Dependencias Documentales

- Requerimiento técnico del proyecto.
- Modelo de amenazas.
- Arquitectura arc42.
- Matriz de compatibilidad.
- Catálogo de comandos críticos.
- Plan de validación SITL.
- ADRs iniciales.
- MVP 0.1: `mvp-0.1-alcance-y-criterios.md`.
- Regla testeable: `especificacion-arm-auto-001.md`.
