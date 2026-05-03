# Especificación de Configuración

## 1. Objetivo

Definir el archivo de configuración del gateway, sus validaciones y valores seguros. La configuración debe evitar ambigüedades operativas y no debe mezclar secretos con parámetros ordinarios cuando pueda evitarse.

## 2. Formato Inicial

Formato recomendado: TOML.

Ruta CLI:

```bash
mavlink-shield-gateway --config config.toml
```

## 3. Ejemplo Inicial

```toml
[transport]
mode = "udp"

[udp]
listen_gcs = "0.0.0.0:14551"
listen_vehicle = "0.0.0.0:14550"
vehicle_addr = "127.0.0.1:14540"
gcs_addr = "127.0.0.1:14552"
read_timeout_ms = 100
max_datagram_size = 2048

[serial]
port = "/dev/ttyACM0"
baud_rate = 57600
data_bits = 8
parity = "none"
stop_bits = 1
read_timeout_ms = 100

[security]
certified_ips = ["127.0.0.1"]
unknown_mode_policy = "block"
audit_only = false
block_arm_in_auto_mode = true

[signing]
policy = "observe"
link_id = 0
# key_path = "/ruta/local/segura/mavlink-signing-test.key"

[crypto]
enabled = false
algorithm = "chacha20poly1305"
key_source = "env"
key_env = "MAVLINK_SHIELD_KEY"

[logging]
level = "info"
payload_logging = false

[metrics]
enabled = true
readonly_bind = "127.0.0.1:14600"
```

## 4. Validaciones Obligatorias

- `transport.mode` debe ser `udp` o `serial`.
- En modo `udp`, direcciones UDP requeridas deben ser válidas.
- En modo `serial`, `port` y `baud_rate` deben estar presentes.
- `unknown_mode_policy` debe ser `block`, `allow` o `audit_only`.
- `signing.policy` debe ser `observe`, `audit` o `enforce`; `audit` y
  `enforce` requieren `signing.key_path`.
- `signing.key_path`, cuando exista, debe ser una ruta absoluta local hacia un
  archivo regular de clave hexadecimal de 32 bytes, no debe ser symlink, no debe
  estar dentro del worktree Git actual del gateway y debe tener permisos
  restrictivos. En Unix, el propietario debe coincidir con el usuario efectivo
  del proceso gateway.
- `payload_logging = true` debe requerir confirmación explícita de entorno no productivo.
- Si `crypto.enabled = true`, debe existir clave válida.
- La clave no debe aparecer como literal en logs.
- `max_datagram_size` debe tener límites razonables.
- `metrics.readonly_bind`, cuando exista, debe ser un socket TCP loopback. Por
  defecto no se abre endpoint read-only.

## 5. Configuración Insegura a Rechazar o Advertir

- `unknown_mode_policy = "allow"` con comandos críticos habilitados.
- `crypto.enabled = true` contra GCS estándar sin receptor compatible declarado.
- `certified_ips = ["0.0.0.0/0"]` o equivalente amplio.
- `signing.policy = "audit"` o `"enforce"` sin `signing.key_path` seguro.
- `signing.key_path` dentro del repositorio actual, en `target/`,
  `implementacion/`, `definicion/`, `producto/` o cualquier otra carpeta del
  mismo worktree Git.
- `payload_logging = true` en perfil productivo.
- claves en archivo versionado.
- `metrics.readonly_bind` expuesto en direcciones no loopback.

## 6. Perfiles Recomendados

### 6.1 `sitl-dev`

- UDP local.
- `unknown_mode_policy = "block"`.
- `audit_only = false`.
- `signing.policy = "observe"` por defecto; `audit` solo con clave local de
  laboratorio; `enforce` solo en laboratorio controlado y para comandos
  críticos/high-risk según ADR 0010.
- cifrado deshabilitado salvo test específico.
- logging `info`.

### 6.2 `security-research`

- UDP local o red controlada.
- posibilidad de `audit_only`.
- payload logging solo si no hay datos sensibles.
- métricas detalladas.

### 6.3 `lab-serial`

- Serial habilitado.
- sin comandos reales peligrosos salvo procedimiento explícito.
- logging de seguridad persistente.

## 7. Preguntas Pendientes

- ¿Se permitirá CIDR en `certified_ips` o solo IP exacta?
- ¿Se añadirá schema JSON/TOML formal?
- ¿Se soportarán perfiles múltiples en un único archivo?
- ¿Cómo se declarará que existe receptor cifrado compatible?
