# Verificación del Entorno de Pruebas MVP 0.1

## 1. Objetivo

Verificar si el proyecto dispone del entorno necesario para validar MVP 0.1:

- implementación Rust;
- tests unitarios e integración;
- fuzzing;
- análisis de dependencias;
- ArduPilot Copter SITL;
- MAVProxy;
- QGroundControl;
- soporte documental mediante skills y MCP.

Esta verificación no incluye operación con hardware real.

## 2. Resultado Ejecutivo

Estado general:

- Entorno Rust: listo.
- Herramientas de calidad Rust: listas.
- Fuzzing: listo a nivel herramienta.
- ArduPilot SITL: clonado y ArduCopter compilado.
- MAVProxy: listo en venv local.
- QGroundControl: AppImage disponible.
- GitHub CLI: autenticado.
- Skills/MCP: listos.
- Proyecto Rust: pendiente de crear `Cargo.toml` y estructura inicial.
- Validación SITL end-to-end: pendiente de ejecutar una vez exista gateway.

Conclusión:

El entorno está preparado para iniciar la implementación del esqueleto Rust MVP 0.1 y para preparar tests. La validación completa `SITL -> Gateway -> QGroundControl` dependerá de crear el gateway y ejecutar los procesos desde una terminal normal con red local permitida.

## 3. Rust y Calidad

Verificado:

| Herramienta | Estado |
|---|---|
| `rustc` | `1.95.0` |
| `cargo` | `1.95.0` |
| `rustfmt` | disponible |
| `clippy` | disponible |
| `cargo-audit` | `0.22.1` |
| `cargo-deny` | `0.19.4` |
| `cargo-nextest` | `0.9.133` |
| `cargo-fuzz` | `0.13.1` |
| Rust nightly | instalado |

Limitación:

- Aún no hay `Cargo.toml`, por lo que no se han podido ejecutar `cargo fmt`, `cargo clippy`, `cargo test`, `cargo audit`, `cargo deny` o `cargo nextest` sobre el proyecto real.

## 4. ArduPilot SITL

Verificado:

| Elemento | Estado |
|---|---|
| Repositorio ArduPilot | clonado en `tools/ardupilot` |
| Commit ArduPilot | `8530bf14d1` |
| Submódulos | inicializados |
| `sim_vehicle.py` | responde a `--help` |
| ArduCopter SITL | compilado |
| Binario | `tools/ardupilot/build/sitl/bin/arducopter` |

Observación:

- La compilación de `bin/arducopter` terminó correctamente.
- Un arranque desde el sandbox falló al abrir sockets por restricción del entorno.
- El usuario confirmó arranque desde terminal normal, con ventana SITL esperando conexión.

## 5. MAVProxy

Verificado:

| Elemento | Estado |
|---|---|
| Venv local | `tools/mavproxy-venv` |
| MAVProxy | `1.8.74` |
| `pymavlink` | `2.4.49` |
| `empy` | `3.3.4` |
| `setuptools` | fijado a `<81` para `pkg_resources` |

Comando:

```bash
tools/mavproxy-venv/bin/mavproxy.py --version
```

Nota:

- MAVProxy avisa de posible conflicto con ModemManager. Para MVP 0.1 SITL/UDP no bloquea. Para hardware real requeriría análisis aparte.

## 6. QGroundControl

Verificado:

| Elemento | Estado |
|---|---|
| AppImage | `tools/QGroundControl-x86_64.AppImage` |
| Ejecutable | sí |
| `--appimage-help` | correcto |
| Arranque gráfico | no verificado por Codex |

Comando:

```bash
./tools/QGroundControl-x86_64.AppImage
```

## 7. GitHub, Skills y MCP

GitHub:

- `gh` autenticado como `ArgosML-tech`.
- Scopes: `gist`, `read:org`, `repo`, `workflow`.

MCP:

- `openaiDeveloperDocs`: habilitado.
- `rustDocs`: habilitado.

Skills instaladas:

- `mavlink-rust-shield-gateway`.
- `rust-security-mvp`.
- `mavlink-sitl-testing`.

## 8. Procesos y Puertos

Desde el sandbox:

- No se detectaron procesos activos filtrados de `arducopter`, `sim_vehicle.py`, `mavproxy.py` o `QGroundControl`.
- No se pudieron inspeccionar sockets con `ss` por restricción del sandbox: `Operation not permitted`.

Implicación:

- La validación de puertos debe realizarse desde terminal normal.

Comando recomendado fuera del sandbox:

```bash
ss -ltnup | grep -E '5760|14540|14550|14551|14552|9005'
```

## 9. Preparación para Validaciones MVP

### 9.1 Validaciones Listas para Ejecutarse Cuando Exista Código

- Tests unitarios de configuración.
- Tests unitarios de clasificación de modo.
- Tests unitarios de `ARM-AUTO-001`.
- Tests unitarios de `CRITICAL-UNKNOWN-001`.
- Tests de parse error.
- `cargo fmt`.
- `cargo clippy`.
- `cargo test`.
- `cargo nextest run`.
- `cargo audit`.
- `cargo deny check`.
- Fuzzing inicial.

### 9.2 Validaciones Pendientes de Implementación

- Proxy UDP bidireccional.
- Recepción de `HEARTBEAT`.
- Extracción de `custom_mode`.
- Clasificación `custom_mode == 3` como `Automatic`.
- Logs estructurados.
- Métricas mínimas.
- Test harness de cifrado.
- End-to-end SITL con gateway.

## 10. Próximo Paso Recomendado

Crear el esqueleto Rust del MVP:

- `Cargo.toml`;
- `src/main.rs`;
- módulos `config`, `transport`, `mavlink_codec`, `flight_state`, `security_filter`, `metrics`, `logging`;
- tests unitarios iniciales para `validacion-modo-ardupilot-sitl.md` y `especificacion-arm-auto-001.md`.

Después de eso se podrá ejecutar la primera batería real:

```bash
. "$HOME/.cargo/env"
cargo fmt
cargo clippy
cargo test
cargo nextest run
cargo audit
cargo deny check
```

