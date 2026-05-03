# Herramientas para Implementación MVP 0.1

## 1. Objetivo

Definir qué herramientas locales, skills y MCP servers conviene instalar o preparar antes de implementar MVP 0.1 de MAVLink-Rust Shield Gateway.

El criterio principal es seguridad y foco: instalar lo necesario para Rust, pruebas, fuzzing, documentación y SITL, evitando herramientas que puedan operar drones reales o ampliar demasiado el acceso del asistente.

## 2. Estado Detectado en el Entorno Actual

Comprobación realizada desde el workspace:

Disponible:

- `node` v24.15.0.
- `npm` 11.12.1.
- `npx`.
- `python3` 3.12.3.
- `git`.
- `codex`.
- MCP global `openaiDeveloperDocs` registrado.
- MCP `rustDocs` registrado con `npx -y @iflow-mcp/rust-docs-mcp-server`.
- Skill instalada `mavlink-rust-shield-gateway`.
- Skills instaladas `rust-security-mvp` y `mavlink-sitl-testing`.
- Rust stable instalado mediante `rustup`: `rustc 1.95.0`.
- Rust nightly instalado para fuzzing.
- `cargo-audit` 0.22.1.
- `cargo-deny` 0.19.4.
- `cargo-nextest` 0.9.133.
- `cargo-fuzz` 0.13.1.
- VS Code: `rust-analyzer`, `CodeLLDB`, `Even Better TOML`, `crates`.
- VS Code GitHub: Pull Requests y Actions.
- GitHub CLI autenticado como `ArgosML-tech`.
- QGroundControl AppImage en `tools/QGroundControl-x86_64.AppImage`.
- `cmake` 3.28.3.
- `clang` 18.1.3.
- `lld`.
- `python3.12-venv` funcional.
- MAVProxy 1.8.74 instalado en `tools/mavproxy-venv/`.
- `pymavlink` 2.4.49 instalado en el venv de MAVProxy.
- ArduPilot clonado en `tools/ardupilot/`.
- `sim_vehicle.py` disponible en `tools/ardupilot/Tools/autotest/sim_vehicle.py`.
- ArduCopter SITL compilado en `tools/ardupilot/build/sitl/bin/arducopter`.

No encontrado en el PATH actual:

- `QGroundControl` como comando global.

Implicación:

- Ya es posible compilar y probar el gateway Rust si el shell carga `$HOME/.cargo/env`.
- Ya existe MAVProxy local en venv y ArduPilot SITL está clonado/compilado.
- La fase de implementación puede comenzar con esqueleto Rust, tests unitarios y fuzzing.
- La ejecución SITL desde dentro del sandbox puede fallar al abrir sockets; para validación real, ejecutar desde terminal normal o con permisos adecuados.

Para shells existentes:

```bash
. "$HOME/.cargo/env"
```

## 3. Herramientas Obligatorias para MVP 0.1

### 3.1 Rust Toolchain

Instalar:

- `rustup`.
- toolchain `stable`.
- componentes `rustfmt`, `clippy`, `rust-src`.

Comandos esperados:

```bash
rustup default stable
rustup component add rustfmt clippy rust-src
```

Validación:

```bash
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
```

Motivo:

- MVP 0.1 es una CLI Rust.
- `rustfmt`, `clippy` y `cargo test` son parte del Definition of Done.
- `rust-src` mejora soporte de `rust-analyzer`.

Estado:

- Instalado mediante `rustup` en el usuario.

### 3.2 Tooling Base de Build en Linux

Instalar:

- compilador C/C++.
- linker.
- `pkg-config`.
- dependencias básicas de desarrollo.

En Ubuntu/Debian normalmente:

```bash
sudo apt install build-essential pkg-config
```

Motivo:

- algunas crates pueden compilar dependencias nativas;
- fuzzing y herramientas cargo requieren toolchain nativa.

Estado:

- `gcc`, `g++`, `make`, `pkg-config`, `cmake`, `clang` y `lld` están disponibles.

### 3.3 VS Code

Instalar o verificar extensiones:

- `rust-analyzer`.
- `CodeLLDB`.
- `Even Better TOML`.
- `crates` o alternativa equivalente para inspección de dependencias.

Motivo:

- `rust-analyzer` soporta navegación, autocompletado y diagnósticos;
- `CodeLLDB` permite depurar parsing y decisiones de política;
- TOML será usado para configuración y `Cargo.toml`.

Estado:

- Instaladas.

## 4. Herramientas Recomendadas de Calidad Rust

### 4.1 cargo-nextest

Instalar:

```bash
cargo install --locked cargo-nextest
```

Uso:

```bash
cargo nextest run
```

Motivo:

- runner de tests más rápido y útil cuando crezca la matriz de tests.

Prioridad:

- Media para el primer esqueleto.
- Alta cuando existan varios módulos y tests.

Estado:

- Instalado.

### 4.2 cargo-audit

Instalar:

```bash
cargo install cargo-audit --locked
```

Uso:

```bash
cargo audit
```

Motivo:

- revisar `Cargo.lock` contra RustSec Advisory Database.
- importante por el perfil security-sensitive del proyecto.

Prioridad:

- Alta antes de fijar dependencias de MVP.

Estado:

- Instalado.

### 4.3 cargo-deny

Instalar:

```bash
cargo install --locked cargo-deny
```

Uso:

```bash
cargo deny init
cargo deny check
```

Motivo:

- revisar licencias, crates baneadas, duplicados, fuentes y advisories.
- complementa `cargo-audit` con control de supply chain.

Prioridad:

- Media al inicio.
- Alta antes de publicar o abrir CI.

Estado:

- Instalado.

## 5. Fuzzing

### 5.1 cargo-fuzz

Instalar:

```bash
rustup install nightly
cargo install cargo-fuzz
```

Uso inicial:

```bash
cargo fuzz init
cargo fuzz add fuzz_mavlink_frame
```

Notas:

- `cargo-fuzz` usa libFuzzer.
- Requiere nightly para flags de sanitización.
- Requiere compilador C++ con soporte C++11.

Prioridad:

- Media durante el primer esqueleto.
- Alta cuando existan `mavlink_codec`, `security_filter` y parsing de configuración.

Estado:

- Rust nightly instalado.
- `cargo-fuzz` instalado.

## 6. SITL y GCS

### 6.1 ArduPilot SITL

Instalar/preparar:

- repositorio ArduPilot;
- entorno de build ArduPilot para Linux;
- `sim_vehicle.py`;
- MAVProxy.

Validación esperada:

```bash
sim_vehicle.py --help
mavproxy.py --version
```

Uso esperado en MVP:

```bash
sim_vehicle.py -v copter --console --map -w
```

Motivo:

- MVP 0.1 valida ArduPilot Copter SITL.
- `custom_mode` de `HEARTBEAT` se valida contra SITL.

Prioridad:

- Alta antes de pruebas de integración.
- No imprescindible para crear el esqueleto Rust inicial.

Estado:

- Instalado para MVP.
- ArduPilot clonado en `tools/ardupilot`.
- `sim_vehicle.py --help` funciona.
- ArduCopter SITL compiló correctamente.
- MAVProxy 1.8.74 está instalado en `tools/mavproxy-venv/bin/mavproxy.py`.
- `setuptools` quedó fijado a `<81` en el venv para compatibilidad con `pkg_resources`.
- Un arranque directo del binario SITL dentro del sandbox falló al abrir sockets con `Operation not permitted`; esto es una restricción del entorno, no del build.

Comando local:

```bash
tools/mavproxy-venv/bin/mavproxy.py --version
tools/ardupilot/Tools/autotest/sim_vehicle.py --help
```

### 6.2 QGroundControl

Instalar:

- QGroundControl para Linux, preferiblemente desde fuente oficial.

Validación:

- la aplicación abre;
- puede crear conexión UDP al puerto configurado del gateway;
- no se conecta directamente a SITL durante pruebas del gateway.

Prioridad:

- Alta para validación MVP 0.1.

Estado:

- AppImage oficial descargado en `tools/QGroundControl-x86_64.AppImage`.
- El AppImage responde a `--appimage-help`.
- No se verificó arranque gráfico.

Comando local:

```bash
./tools/QGroundControl-x86_64.AppImage
```

## 7. MCP Servers Recomendados

### 7.1 Mantener: openaiDeveloperDocs

Estado:

- ya registrado.

Uso:

- documentación oficial OpenAI/Codex/MCP si se necesita ajustar skills, MCP o flujos Codex.

### 7.2 Añadir: Rust/docs.rs MCP

Prioridad:

- Alta.

Uso:

- consultar APIs actuales de `mavlink`, `tokio`, `serde`, `tracing`, `clap`, `chacha20poly1305`, `zeroize`, `criterion` y `cargo-fuzz`.

Razón:

- reduce errores de API durante implementación.
- evita depender de memoria o ejemplos obsoletos.

Opciones a evaluar:

- `rust-docs-mcp-server`.
- `mcp-docsrs`.
- `cratedocs-mcp`.

Nota:

- ya se había observado que alguna opción requiere Bun/Node o autenticación GitHub. Conviene instalar solo una opción estable y limitada.

Estado:

- Registrado `rustDocs` usando `npx -y @iflow-mcp/rust-docs-mcp-server`.

### 7.3 Añadir si el repo se sube a GitHub: GitHub MCP / plugin

Prioridad:

- Media ahora.
- Alta cuando el proyecto tenga repositorio GitHub y CI.

Uso:

- revisar PRs;
- consultar issues;
- inspeccionar CI;
- publicar cambios;
- revisar dependencias y ejemplos.

Estado:

- el plugin GitHub está disponible en esta sesión.
- `gh` está instalado y autenticado como `ArgosML-tech`.
- el token tiene scopes `gist`, `read:org`, `repo` y `workflow`.
- el workspace actual no parece estar inicializado como repositorio Git.

### 7.4 Opcional: Filesystem MCP Limitado

Prioridad:

- Baja en esta sesión, porque Codex ya accede al workspace.

Condición:

- exponer solo el directorio del proyecto.
- no exponer `$HOME`, `.ssh`, claves, tokens, directorios de QGC ni configuraciones sensibles.

### 7.5 No Instalar para MVP 0.1: MAVLink MCP Operacional

Motivo:

- puede enviar comandos MAVLink.
- añade riesgo operacional innecesario.

Permitido solo como referencia o futuro MCP propio:

- simulación por defecto;
- read-only por defecto;
- fixtures MAVLink;
- sin hardware real;
- sin comandos críticos salvo test explícito contra SITL.

## 8. Skills Recomendadas

### 8.1 Mantener Skill del Proyecto

Instalada:

- `mavlink-rust-shield-gateway`.

Acción pendiente opcional:

- sincronizar la copia instalada en `$CODEX_HOME/skills` con la copia versionada del proyecto si se quiere que recoja el contexto actualizado de MVP 0.1.

Estado:

- Sincronizada con la copia versionada actual.

### 8.2 Crear o Instalar Skill Rust Security MVP

Prioridad:

- Alta si se va a implementar con Codex de forma intensiva.

Estado:

- Creada en `definicion/codex-package/skills/rust-security-mvp/`.
- Instalada en `$CODEX_HOME/skills/rust-security-mvp/`.

Contenido recomendado:

- patrones Rust para errores sin `panic!`;
- `thiserror`/`anyhow` según capa;
- `tokio` y cancelación ordenada;
- `tracing`;
- tests unitarios e integración;
- fuzzing con `cargo-fuzz`;
- revisión de dependencias con `cargo-audit`/`cargo-deny`.

### 8.3 Crear o Instalar Skill MAVLink Testing

Prioridad:

- Media.

Estado:

- Creada en `definicion/codex-package/skills/mavlink-sitl-testing/`.
- Instalada en `$CODEX_HOME/skills/mavlink-sitl-testing/`.

Contenido recomendado:

- fixtures MAVLink;
- `HEARTBEAT.custom_mode`;
- `COMMAND_LONG`;
- `COMMAND_INT`;
- `MAV_CMD_COMPONENT_ARM_DISARM`;
- ArduPilot SITL;
- QGroundControl UDP.

Restricción:

- simulación por defecto;
- prohibido hardware real.

### 8.4 Crear o Instalar Skill Code Review Security

Prioridad:

- Media.

Contenido recomendado:

- checklist de review para input no confiable;
- logging sin secretos;
- backpressure;
- latencia;
- parsing y errores;
- tests negativos.

## 9. Herramientas No Recomendadas al Inicio

No instalar para MVP 0.1:

- MCP o skills que armen, despeguen, aterricen o cambien modo en drones reales.
- Herramientas de control DJI/Tello.
- MCP filesystem con acceso amplio a `$HOME`.
- Secret managers complejos antes de tener módulo de configuración.
- Grafana/Prometheus antes de estabilizar métricas internas.
- PX4 toolchain completa antes de terminar ArduPilot SITL MVP.
- Mission Planner en esta fase, salvo para compatibilidad posterior.

## 10. Orden Recomendado de Instalación

### Bloque 1: Implementación Rust

1. `rustup` + stable toolchain.
2. `rustfmt`, `clippy`, `rust-src`.
3. `build-essential`, `pkg-config`.
4. VS Code `rust-analyzer` y `CodeLLDB`.

### Bloque 2: Calidad y Seguridad

5. `cargo-audit`.
6. `cargo-deny`.
7. `cargo-nextest`.

### Bloque 3: Fuzzing

8. nightly Rust.
9. `cargo-fuzz`.

### Bloque 4: Validación SITL

10. ArduPilot SITL.
11. MAVProxy.
12. QGroundControl.

### Bloque 5: Asistencia Documental

13. Rust/docs.rs MCP.
14. GitHub MCP/plugin cuando haya repo GitHub.
15. Skills técnicas específicas si no se quiere depender solo de la skill del proyecto.

## 11. Comandos de Comprobación Final

```bash
rustc --version
cargo --version
cargo fmt --version
cargo clippy --version
cargo audit --version
cargo deny --version
cargo nextest --version
cargo fuzz --help
sim_vehicle.py --help
mavproxy.py --version
codex mcp list
```

## 12. Fuentes Revisadas

- Rust/Cargo installation.
- rust-analyzer installation.
- Clippy installation.
- Rust Fuzz Book: `cargo-fuzz`.
- RustSec `cargo-audit`.
- cargo-deny documentation.
- cargo-nextest documentation.
- ArduPilot SITL documentation.
- QGroundControl official downloads/configuration.
