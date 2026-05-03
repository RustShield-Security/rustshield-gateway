# Plan de Fuzzing MAVLink

## 1. Objetivo

Detectar fallos de robustez y seguridad en componentes que procesan input no confiable: parser, clasificador, filtro, configuración y envoltorio cifrado.

## 2. Objetivos de Fuzzing

- No `panic!`.
- No consumo excesivo no acotado.
- No decisiones inconsistentes para comandos críticos.
- No aceptación silenciosa de frames inválidos como comandos válidos.
- No fallos criptográficos por formatos malformados.

## 3. Targets Iniciales

| Target | Entrada | Propiedad |
|---|---|---|
| `fuzz_mavlink_frame` | bytes arbitrarios | parsear o rechazar sin crash |
| `fuzz_command_classifier` | mensajes MAVLink construidos/mutados | clasificación estable |
| `fuzz_security_policy` | mensaje + estado + origen | decisión válida |
| `fuzz_config` | TOML arbitrario | validar o rechazar sin crash |
| `fuzz_crypto_wrapper` | bytes cifrados/mutados | descifrar o rechazar |

## 4. Corpus Inicial

- `HEARTBEAT` válido.
- `COMMAND_LONG` con ARM.
- `COMMAND_INT` con ARM.
- frame truncado.
- frame con checksum inválido.
- frame con `message_id` desconocido.
- payload máximo razonable.
- wrapper cifrado válido.
- wrapper cifrado con tag manipulado.

## 4.1 Corpus de Fase 0016

El corpus de `fuzz_mavlink_frame` se genera de forma determinista con:

```bash
cargo run --bin export-fuzz-corpus
```

Fixtures cubiertos:

- MAVLink v1 válido;
- MAVLink v2 válido;
- MAVLink v2 firmado observado sin validación criptográfica;
- comandos y mensajes críticos del catálogo inicial (`COMMAND_LONG`,
  `COMMAND_INT`, `MISSION_COUNT`, `PARAM_SET`, `MANUAL_CONTROL`);
- frame truncado;
- frame con bytes sobrantes;
- frame con checksum alterado.

## 5. Invariantes

- Un paquete inválido no produce `allow`.
- Un comando crítico en modo desconocido no produce `allow` salvo política explícita.
- Una clave o nonce inválido no se acepta silenciosamente.
- Errores no contienen secretos.

## 6. Herramienta Recomendada

- `cargo-fuzz` con libFuzzer.

## 7. Criterios de Salida

- Targets principales ejecutan durante ventana mínima definida sin crash.
- Cualquier crash produce issue o test de regresión.
- Hallazgos que afecten seguridad generan ADR o actualización de política si cambian comportamiento.

Resultado fase 0016:

- `ASAN_OPTIONS=detect_leaks=0 cargo +nightly fuzz run fuzz_mavlink_frame -- -runs=5000`
  completó sin crash lógico.
- Una propiedad demasiado amplia del target fue corregida: en modo conocido el
  gateway no promete bloquear todo comando catalogado; esa cobertura depende de
  reglas explícitas. La invariante conservadora se mantiene para modo
  desconocido.

## 4.2 Corpus y Resultado de Fase 0021C-3

El corpus determinista de `fuzz_mavlink_frame` se amplio con fixtures de
seguridad pre-hardware:

- MAVLink v1 y v2 validos;
- MAVLink v2 firmado con clave insegura de test;
- firma alterada, `link_id` inesperado y replay-like;
- frame truncado, checksum alterado, bytes sobrantes y probe de limite;
- `SETUP_SIGNING` como mensaje sensible;
- `COMMAND_LONG`/`COMMAND_INT` para armado, force-arm, takeoff, land,
  set-mode, mission start, reposition y reboot;
- mensajes catalogados high-risk: mission mutation, `PARAM_SET`,
  `MANUAL_CONTROL` y `RC_CHANNELS_OVERRIDE`.

El target tambien valida que la ruta de signing no muta ni reserializa el
datagrama durante la validacion.

Resultado fase 0021C-3:

- Evidencia:
  `implementacion/evidencias/fuzz-20260503T062901Z/README.md`.
- `cargo fmt --check`: correcto.
- `cargo clippy --all-targets --all-features -- -D warnings`: correcto.
- `cargo test`: 90 tests pasados.
- `ASAN_OPTIONS=detect_leaks=0 cargo +nightly fuzz run fuzz_mavlink_frame -- -runs=20000`:
  completado sin crash.
- `ASAN_OPTIONS=detect_leaks=0 cargo +nightly fuzz run fuzz_mavlink_frame -- -max_total_time=180`:
  2455113 runs en 181 segundos, sin crash.
- Corpus total tras la campaña local: 2341 entradas.
- No se generaron artifacts de crash.
- La campaña de 1800 segundos queda como recomendacion para ejecucion nocturna
  o CI dedicado; esta fase no valida hardware ni cobertura exhaustiva de
  dialectos MAVLink.
