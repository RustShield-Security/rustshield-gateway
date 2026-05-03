# Modelo de Amenazas: MAVLink-Rust Shield Gateway

## 1. Estado y Alcance

- Estado: borrador inicial.
- Método base: preguntas OWASP de threat modeling.
- Alcance: gateway Rust, configuración, transportes UDP/Serial, parser MAVLink, filtro de comandos, estado de vuelo, cifrado, logging y métricas.
- Fuera de alcance: compromiso físico del autopiloto, vulnerabilidades internas de la GCS, seguridad completa de la red, defensa contra adversarios con control del host donde corre el gateway.

## 2. Qué Estamos Construyendo

Un proxy MAVLink 2.0 que se coloca entre una GCS y un vehículo o simulador. El gateway recibe paquetes desde ambos lados, parsea los mensajes, mantiene estado de vuelo conocido, aplica políticas sobre comandos críticos y reenvía o bloquea según corresponda. Puede cifrar flujos configurados hacia un receptor compatible.

## 3. Activos a Proteger

- Seguridad del vehículo o simulador ante comandos no autorizados.
- Integridad de comandos permitidos.
- Disponibilidad del flujo de telemetría.
- Confidencialidad e integridad de flujos cifrados.
- Material criptográfico.
- Configuración de políticas y allowlists.
- Evidencia de auditoría en logs y métricas.
- Compatibilidad con routing y signing MAVLink.

## 4. Límites de Confianza

```text
GCS / red local no confiable
        |
        v
UDP socket del gateway
        |
        v
Parser MAVLink y filtro de seguridad
        |
        v
Transporte hacia vehículo o simulador
        |
        v
Autopiloto / SITL
```

Límites relevantes:

- Todo paquete recibido por UDP es no confiable.
- La IP de origen es una señal débil, no una identidad criptográfica.
- Todo frame MAVLink debe validarse antes de usar sus campos.
- La configuración local es confiable solo si sus permisos y origen están controlados.
- El host donde corre el gateway es parte de la base de confianza inicial.

## 5. Suposiciones de Seguridad

- La primera validación se realiza en SITL.
- El gateway corre en un host controlado por el equipo.
- La GCS puede comportarse de forma incorrecta, accidental o maliciosa.
- El enlace UDP puede recibir tráfico inyectado o repetido.
- El autopiloto puede aceptar comandos MAVLink válidos si llegan a él.
- No se asume que MAVLink signing esté siempre activado.
- No se asume que la GCS estándar pueda descifrar tráfico cifrado externo.

## 6. Amenazas Principales

### T-001: Inyección de Comando de Armado

Un origen no autorizado envía `MAV_CMD_COMPONENT_ARM_DISARM` mientras el vehículo está en modo automático.

- Impacto: alto.
- Probabilidad en red local de pruebas: media.
- Mitigación inicial: regla `ARM-AUTO-001`, allowlist, logs, métricas.
- Riesgo residual: la IP puede ser falsificada; se requiere autenticación fuerte en fases posteriores.

### T-002: Spoofing UDP

Un atacante envía datagramas desde una IP falsificada o desde un host no previsto.

- Impacto: alto.
- Probabilidad: media.
- Mitigación inicial: bind restrictivo, allowlist, rechazo de orígenes desconocidos, modo conservador.
- Riesgo residual: IP allowlist no equivale a autenticación.

### T-003: Replay de Comandos

Un comando MAVLink válido capturado se reenvía posteriormente.

- Impacto: alto para comandos críticos.
- Probabilidad: media.
- Mitigación inicial: logs y catálogo de comandos críticos.
- Mitigación futura: MAVLink signing validado, ventana temporal, contador, nonce o canal autenticado.
- Riesgo residual: alto si no hay signing ni control antireplay.

### T-004: Caída por Paquete Malformado

Frames corruptos o incompletos provocan `panic!`, consumo excesivo o salida del proceso.

- Impacto: alto sobre disponibilidad.
- Probabilidad: alta.
- Mitigación: parser robusto, manejo de errores recuperables, fuzzing, límites de buffer, tests negativos.
- Riesgo residual: bugs de crate o integración.

### T-005: Confusión de Routing MAVLink

El gateway reenvía comandos al `target_system` o `target_component` incorrecto, o modifica campos que cambian semántica.

- Impacto: alto.
- Probabilidad: media.
- Mitigación: preservar mensajes por defecto, documentar routing, tests con destinos concretos y broadcast.
- Riesgo residual: redes MAVLink complejas con múltiples vehículos o componentes.

### T-006: Ruptura de MAVLink Signing

El gateway reserializa, modifica o envuelve mensajes firmados de forma incompatible.

- Impacto: alto sobre compatibilidad y seguridad.
- Probabilidad: media si se activa signing.
- Mitigación: ADR 0008, modo transparente para paquetes firmados, pruebas con signing cuando sea posible.
- Riesgo residual: alto hasta validar con GCS/autopiloto concretos.

### T-007: Exposición de Claves

Una clave de cifrado se imprime en logs, queda en un crash dump, se guarda con permisos débiles o se versiona accidentalmente.

- Impacto: alto.
- Probabilidad: media.
- Mitigación: variables de entorno o fichero con permisos restrictivos, no logging de secretos, `zeroize` cuando sea razonable, revisión de configuración.
- Riesgo residual: compromiso del host.

### T-008: Nonce Reutilizado en AEAD

ChaCha20-Poly1305 se usa con nonce repetido para la misma clave.

- Impacto: crítico.
- Probabilidad: media si el diseño del envoltorio no es explícito.
- Mitigación: especificación criptográfica, contador persistente o nonce aleatorio con probabilidad controlada, tests.
- Riesgo residual: reinicios y concurrencia si no se diseñan bien.

### T-009: Falsa Compatibilidad por Cifrado Directo

Se cifra tráfico hacia una GCS estándar que espera MAVLink plano, rompiendo operación o induciendo una falsa sensación de protección.

- Impacto: medio-alto.
- Probabilidad: alta si no se documenta.
- Mitigación: configuración explícita, ADR, matriz de compatibilidad, receptor compatible.
- Riesgo residual: errores de despliegue.

### T-010: Denegación de Servicio por Alto Volumen

Un emisor genera demasiados paquetes, satura CPU, memoria, logs o canales internos.

- Impacto: alto.
- Probabilidad: media.
- Mitigación: límites de buffer, backpressure, rate limits para logs, métricas, benchmarks.
- Riesgo residual: hardware limitado o tráfico extremo.

### T-011: Logs con Información Sensible

Logs incluyen claves, payloads sensibles, coordenadas o detalles operativos innecesarios.

- Impacto: medio-alto.
- Probabilidad: media.
- Mitigación: política de logging, redacción, niveles, campos permitidos.
- Riesgo residual: modo debug mal usado.

### T-012: Política Ante Estado Desconocido Insegura

Antes de recibir `HEARTBEAT`, el gateway permite comandos críticos que debería bloquear.

- Impacto: alto.
- Probabilidad: media.
- Mitigación: `block` por defecto para críticos en estado desconocido.
- Riesgo residual: impacto operativo si bloquea demasiado en arranque.

## 7. Controles Iniciales

- Deny-by-default para comandos críticos cuando el estado sea desconocido.
- Separación entre telemetría y comandos.
- Allowlist configurable de IPs solo como control inicial.
- Logs estructurados de decisiones de seguridad.
- Métricas de bloqueos, errores y latencia.
- Fuzzing planificado del parser y filtro.
- ADRs para signing, cifrado, routing y política de estado desconocido.

## 8. Requisitos de Prueba Derivados

- Probar armado desde IP no certificada en modo automático.
- Probar armado desde IP certificada en modo automático.
- Probar comando crítico antes de recibir `HEARTBEAT`.
- Probar paquetes MAVLink corruptos.
- Probar tráfico de alto volumen.
- Probar replay simulado de comando crítico.
- Probar configuración criptográfica inválida.
- Probar que logs no contienen claves.
- Probar routing con `target_system` concreto y broadcast.

## 9. Riesgo Residual Aceptado en Primera Versión

La primera versión no elimina ataques de spoofing o replay si no se activa signing o autenticación fuerte. El objetivo aceptable de la primera versión es inspección, bloqueo básico, trazabilidad y validación en SITL. Cualquier despliegue con hardware real requiere una evaluación de riesgo separada.

## 10. Preguntas Abiertas

- ¿Autopiloto objetivo inicial: ArduPilot, PX4 o ambos?
- ¿GCS objetivo inicial: QGroundControl, Mission Planner o ambas?
- ¿En qué fase se implementará validación completa de MAVLink signing?
- ¿El cifrado será gateway-to-gateway o habrá plugin/receptor GCS?
- ¿Qué política exacta se aplicará a comandos críticos distintos de armado?
- ¿Qué datos de telemetría se consideran sensibles en logs?
