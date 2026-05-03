# Plan de Validación SITL Reproducible

## 1. Objetivo

Validar el gateway en simulación antes de considerar hardware real. Este plan cubre conectividad, parsing, estado, políticas, observabilidad, compatibilidad y rendimiento básico.

## 2. Entorno Base

- Sistema operativo prioritario: Linux.
- Simulador objetivo MVP 0.1: ArduPilot SITL.
- GCS objetivo MVP 0.1: QGroundControl.
- Gateway: binario Rust local.
- Transporte MVP 0.1: UDP.

PX4 SITL queda para una fase posterior de compatibilidad.

## 3. Precondiciones

- ArduPilot SITL instalado o disponible mediante entorno de desarrollo controlado.
- QGroundControl instalado.
- Gateway compilado en modo debug o release.
- Archivo `config.toml` de SITL creado.
- No hay hardware real conectado al flujo de pruebas.
- Logging estructurado habilitado.
- Métricas habilitadas.

## 4. Puertos de Referencia

Los puertos exactos pueden cambiar según entorno, pero deben quedar registrados en la evidencia de prueba.

Ejemplo de topología:

```text
ArduPilot SITL 127.0.0.1:14540
        |
        v
Gateway listen_vehicle 127.0.0.1:14550
Gateway listen_gcs     127.0.0.1:14551
        |
        v
QGroundControl UDP     127.0.0.1:14552
```

Regla:

- QGroundControl no debe conectarse directamente al SITL.
- SITL no debe aceptar comandos desde QGroundControl sin pasar por el gateway.

## 5. Topología Inicial

```text
SITL -> UDP -> Gateway -> UDP -> QGroundControl
QGroundControl -> UDP -> Gateway -> UDP -> SITL
```

## 6. Configuración SITL de Ejemplo

La configuración real se mantendrá en un archivo de ejemplo cuando exista código. Para documentación, la forma esperada es:

```toml
[transport]
mode = "udp"

[udp]
listen_vehicle = "127.0.0.1:14550"
listen_gcs = "127.0.0.1:14551"
vehicle_addr = "127.0.0.1:14540"
gcs_addr = "127.0.0.1:14552"
read_timeout_ms = 100
max_datagram_size = 2048

[security]
certified_ips = ["127.0.0.1"]
unknown_mode_policy = "block"
audit_only = false
block_arm_in_auto_mode = true

[crypto]
enabled = false

[logging]
level = "info"
payload_logging = false

[metrics]
enabled = true
```

Para probar bloqueo por IP no certificada en una sola máquina, se debe usar un entorno de red controlado, interfaz alternativa, namespace, contenedor o inyección de datagramas de prueba. No se debe relajar la regla en el código para facilitar el test.

## 7. Procedimiento Paso a Paso

### Paso 1: Arrancar ArduPilot SITL

Objetivo:

- generar tráfico MAVLink sin hardware real.

Evidencia:

- comando usado;
- versión de ArduPilot;
- puerto MAVLink configurado.

### Paso 2: Arrancar Gateway

Objetivo:

- abrir sockets UDP;
- cargar política;
- iniciar logs y métricas.

Evidencia:

- log `gateway.start`;
- log `transport.opened`;
- configuración efectiva no sensible.

### Paso 3: Conectar QGroundControl al Gateway

Objetivo:

- validar que QGC recibe telemetría a través del gateway.

Evidencia:

- captura de conexión;
- heartbeat observado;
- métrica `packets_forwarded_total`.

### Paso 4: Verificar Estado desde HEARTBEAT

Objetivo:

- comprobar que el gateway actualiza `flight_state`.

Evidencia:

- log `flight_state.mode_changed` o estado inicial observado;
- edad del último heartbeat.

### Paso 5: Ejecutar Prueba ARM-AUTO-001

Objetivo:

- demostrar que un armado no certificado en modo automático se bloquea.

Condiciones:

- entorno SITL;
- modo clasificado como automático o estado desconocido controlado;
- origen no certificado.

Evidencia:

- log `security.command_blocked`;
- `rule_id = ARM-AUTO-001` o `CRITICAL-UNKNOWN-001`;
- incremento de `packets_blocked_total`;
- ausencia de reenvío al vehículo.

### Paso 6: Ejecutar Prueba de Origen Certificado

Objetivo:

- demostrar que la política permite un comando cuando el origen y el estado lo permiten.

Condiciones:

- solo simulación;
- origen en `certified_ips`;
- registro explícito del resultado.

Evidencia:

- log `security.command_observed`;
- decisión `allow`;
- incremento de `packets_forwarded_total`.

### Paso 7: Inyectar Paquetes Malformados

Objetivo:

- validar robustez del parser y del pipeline.

Evidencia:

- log `mavlink.parse_error`;
- incremento de `packets_parse_error_total`;
- proceso sigue activo.

### Paso 8: Ejecutar Benchmark Nominal

Objetivo:

- obtener p50/p95/p99 de latencia interna.

Evidencia:

- reporte de benchmark;
- configuración usada;
- hardware usado.

### Paso 9: Revisar Ausencia de Secretos

Objetivo:

- verificar que logs no contienen claves, payload completo ni variables sensibles.

Evidencia:

- muestra de logs;
- revisión manual o test automatizado.

## 8. Casos de Prueba

### SITL-001: Arranque y Conectividad

- Arrancar SITL.
- Arrancar gateway.
- Conectar QGC al gateway.
- Verificar telemetría básica.

Criterio: QGC recibe heartbeat y datos básicos sin conexión directa al simulador.

### SITL-002: Estado desde HEARTBEAT

- Observar `HEARTBEAT`.
- Registrar modo detectado.
- Cambiar modo en simulación si procede.

Criterio: `flight_state` refleja cambios esperados o documenta modo desconocido.

### SITL-003: Bloqueo ARM no Certificado

- Configurar IP no certificada.
- Poner vehículo en modo automático simulado si es seguro.
- Enviar comando de armado desde origen no certificado.

Criterio: el gateway bloquea y SITL no recibe el comando.

### SITL-004: ARM Certificado

- Configurar IP certificada.
- Repetir comando en condiciones controladas de simulación.

Criterio: la política permite el comando si todas las condiciones son válidas.

### SITL-005: Estado Desconocido

- Enviar comando crítico antes de recibir heartbeat válido.

Criterio: bloqueo por `CRITICAL-UNKNOWN-001`.

### SITL-006: Paquetes Malformados

- Inyectar datagramas aleatorios o truncados.

Criterio: no hay crash; se incrementan errores de parseo.

### SITL-007: Cifrado Test Harness

- Activar cifrado solo con test harness aislado en MVP 0.1.

Criterio: el receptor de prueba valida autenticidad y descifra; QGroundControl estándar no se declara compatible si recibe cifrado directo.

## 9. Evidencias Esperadas

- logs de arranque;
- logs de cambio de modo;
- logs de bloqueo;
- métricas de parseo y bloqueo;
- captura de configuración no sensible;
- resultados de latencia nominal.

## 10. Criterios de Salida

- Todos los casos base pasan en al menos una combinación GCS/autopiloto.
- No hay crash con tráfico inválido.
- Los eventos de seguridad son explicables.
- La matriz de compatibilidad se actualiza con resultados reales.
- No hay operación sobre hardware real.
- No se presenta cifrado directo como compatible con QGroundControl estándar.
