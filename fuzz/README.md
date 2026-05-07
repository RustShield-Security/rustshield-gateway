# Fuzzing MVP 0.1

Preparación inicial para `cargo-fuzz`.

## Frontera inicial

La primera frontera estable para fuzzing es:

- función: `mavlink_rust_shield_gateway::mavlink_codec::decode_datagram`;
- entrada: bytes de un datagrama UDP tratado como un único frame MAVLink;
- propiedad mínima: no debe hacer `panic!`; los errores deben quedar expresados
  como `DecodeError` y el proxy debe poder mapearlos a `PARSE-ERROR-001`;
- mensajes prioritarios: `HEARTBEAT`, `COMMAND_LONG`, `COMMAND_INT`;
- contención MVP 0.1: no validar criptográficamente MAVLink signing y no
  reserializar paquetes firmados.

## Corpus determinista

La fase `0016-endurecimiento-routing-fuzzing.md` añade el binario:

```bash
cargo run --bin export-fuzz-corpus
```

Este comando genera seeds nombrados bajo `fuzz/corpus/fuzz_mavlink_frame/`.
El directorio de corpus sigue ignorado por Git para evitar churn binario, pero
los fixtures son reproducibles desde código.

Seeds cubiertos:

- `heartbeat-v1.bin`;
- `heartbeat-v2.bin`;
- `heartbeat-v2-signed.bin`;
- `command-long-arm-v2.bin`;
- `mission-count-v2.bin`;
- `param-set-v2.bin`;
- `manual-control-v2.bin`;
- `command-int-reposition-v2.bin`;
- `truncated-heartbeat-v2.bin`;
- `trailing-heartbeat-v2.bin`;
- `bad-crc-heartbeat-v2.bin`.

## Target inicial

La fase `0007-fuzzing-y-benchmark.md` activa el target:

```bash
cargo fuzz run fuzz_mavlink_frame
```

En este entorno Codex/sandbox puede ser necesario desactivar LeakSanitizer
porque falla bajo `ptrace` aunque el target no haya encontrado un crash:

```bash
ASAN_OPTIONS=detect_leaks=0 cargo +nightly fuzz run fuzz_mavlink_frame -- -runs=1000
```

Propiedades iniciales:

- bytes arbitrarios deben parsearse o rechazarse sin `panic!`;
- los errores de parseo deben mapearse a `DropInvalid`;
- si los bytes producen un mensaje o comando catalogado crítico o de alto
  riesgo, la política con modo de vuelo desconocido no debe producir `Allow`;
- si el frame está marcado como firmado, debe ser MAVLink 2;
- el target no valida signing criptográfico ni reserializa frames firmados.

Campaña corta validada en fase 0016:

```bash
ASAN_OPTIONS=detect_leaks=0 cargo +nightly fuzz run fuzz_mavlink_frame -- -runs=5000
```

Nota: sin `ASAN_OPTIONS=detect_leaks=0`, LeakSanitizer puede fallar bajo el
entorno de ejecución aunque el target complete la campaña sin crash lógico.
