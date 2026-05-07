<div align="center">
  <img src="assets/logo.png" alt="RustShield Gateway logo" width="420">

  # RustShield Gateway

  **MAVLink security gateway research prototype for SITL and controlled laboratory validation.**

  [![Rust](https://img.shields.io/badge/language-Rust-orange.svg?logo=rust)](https://www.rust-lang.org/)
  [![Architecture](https://img.shields.io/badge/architecture-arc42--aligned-blue)](definicion/arquitectura-arc42-mavlink-rust-shield-gateway.md)
  [![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-lightgrey)](#license)
</div>

## What This Is

RustShield Gateway is a Rust-based security gateway for MAVLink-based UAV
systems. It demonstrates semantic command filtering, signing observability,
policy enforcement experiments, structured observability and assurance-oriented
documentation.

This public repository is a **technical preview and showcase**. It is intended
for review, research discussion and partnership conversations. The private
laboratory branch contains additional validation history, internal evidence and
pre-hardware work.

## Current Public Scope

Validated public-preview scope:

- MAVLink v1/v2 parsing using the `common` dialect;
- UDP gateway path for SITL-style deployments;
- ArduPilot Copter SITL as the primary documented target;
- stateful `HEARTBEAT` interpretation for the documented ArduPilot Copter MVP;
- semantic policy checks for critical/high-risk MAVLink commands;
- MAVLink signing observability and laboratory enforcement experiments;
- structured logs and read-only metrics primitives;
- fuzzing, dependency auditing and assurance-oriented documentation.

Public-preview limitations:

- not certified for flight operations;
- not validated for real UAV operation in this public preview;
- not a complete PX4 mode-policy implementation;
- not a complete hardware/radio/Serial product;
- not a replacement for proper key management, network segmentation or platform
  hardening;
- not a guarantee of end-to-end real-time performance.

## Why It Exists

MAVLink is efficient and widely used, but production deployments need explicit
security controls around command authorization, signing, observability and
operational evidence. RustShield explores an external gateway approach that can
sit between a ground station and a vehicle or simulator:

```text
Ground Control Station <-> RustShield Gateway <-> Autopilot / SITL
```

The key idea is semantic filtering: a gateway should not only parse packets, but
also evaluate whether a command is coherent with the current flight context and
security policy.

## Core Capabilities

| Area | Public Preview Capability |
|---|---|
| MAVLink parsing | Parses MAVLink v1/v2 frames and extracts routing/signing metadata. |
| Flight state | Maintains a conservative state from `HEARTBEAT`. |
| Policy engine | Blocks or audits critical/high-risk command classes under documented conditions. |
| Signing | Observes MAVLink 2 signing and supports laboratory validation/enforcement paths. |
| Observability | Emits structured logs and read-only metrics suitable for evidence capture. |
| Assurance docs | Includes ADRs, arc42 architecture, threat model, traceability and risk register. |

## Safety Notice

This repository is for documentation, simulation and controlled laboratory work.
Do not use it to arm, disarm, take off, land, change mode, upload missions,
mutate parameters or send RC override commands to real UAV hardware unless a
separate safety procedure, test plan and explicit authorization exist.

All MAVLink input must be treated as untrusted.

## Quick Start

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Example SITL-oriented scripts are included for public review. They are not a
flight-operation procedure.

```bash
./scripts/run-sitl-gateway.sh
./scripts/send-sitl-arm-command.sh 127.0.0.1:14551
```

## Documentation Map

- [Architecture arc42](definicion/arquitectura-arc42-mavlink-rust-shield-gateway.md)
- [Threat Model](definicion/modelo-amenazas-mavlink-rust-shield-gateway.md)
- [Security Policies](definicion/especificacion-politicas-seguridad.md)
- [Observability Spec](definicion/especificacion-observabilidad.md)
- [Compatibility Matrix](definicion/matriz-compatibilidad-gcs-autopilotos.md)
- [Risk Register](definicion/registro-riesgos.md)
- [Public Scope](docs/public-scope.md)
- [Evidence Summary](docs/evidence-summary.md)
- [Public Roadmap](docs/public-roadmap.md)

## Commercial / Partnership Use

The public repository is intentionally limited. For partnership, integration or
acquisition discussions, the useful artifact is not just the code: it is the
combination of gateway implementation, security policy model, evidence workflow
and controlled laboratory validation plan.

See [Product Brief](docs/product-brief.md).

## Security

Please see [SECURITY.md](SECURITY.md) before reporting vulnerabilities or using
the project in a lab.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
