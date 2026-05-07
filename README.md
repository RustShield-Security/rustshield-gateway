# RustShield Gateway

Security-oriented MAVLink gateway for controlled SITL/laboratory validation.

RustShield Gateway is a Rust-based MAVLink security gateway technical preview
for controlled SITL/laboratory validation. It focuses on semantic command
policy, signing-aware audit/enforce paths, shadow enforcement, read-only
observability and evidence-oriented engineering.

## What It Does

RustShield evaluates selected high-risk MAVLink traffic using:

- semantic command policy for critical/high-risk MAVLink commands;
- conservative flight-state context from `HEARTBEAT`;
- MAVLink signing observe/audit/enforce laboratory paths;
- shadow enforcement for non-blocking impact assessment;
- read-only logs and metrics for evidence capture;
- reproducible local checks and public evidence summaries.

## Current Public Scope

- MAVLink UDP/SITL gateway.
- ArduPilot Copter SITL as the primary documented workflow.
- QGroundControl-oriented laboratory topology.
- Critical/high-risk MAVLink command policy.
- MAVLink signing observe/audit/enforce laboratory validation paths.
- Shadow enforcement counters and events.
- Read-only `/healthz` and `/metrics` observability.
- Public evidence summaries and reproducibility checks.
- Limited PX4 heartbeat fixtures and smoke tests, with PX4 modes treated
  conservatively as `Unknown`.
- Serial transport validated only against virtual PTY devices.

## Not Claimed

- No real UAV flight readiness.
- No certification.
- No hardware/radio validation.
- No production Serial/radio support.
- No complete PX4 mode-policy support.
- No complete MAVLink security coverage.
- No guaranteed end-to-end real-time performance.
- No replacement for platform hardening, key management or network
  segmentation.

## Quick Checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Supply-chain checks used by the project:

```bash
cargo audit
cargo deny check
```

## Public Demo

The public demo is loopback-only and does not require real hardware, radios,
QGroundControl or an autopilot.

```bash
./scripts/run-public-demo.sh
```

See [docs/demo.md](docs/demo.md).

## Evidence

See [docs/evidence/latest/](docs/evidence/latest/) for public, sanitized
evidence summaries.

The public evidence pack is a summary. It is not a certification package and it
does not include private laboratory history, raw internal logs or customer
material.

## Commercial / Lab Pilots

See [COMMERCIAL.md](COMMERCIAL.md) for assessment, laboratory pilot and partner
integration options.

## Documentation

- [Public Scope](docs/public-scope.md)
- [Public Claims](docs/claims.md)
- [Limitations](docs/limitations.md)
- [Responsible Use](docs/responsible-use.md)
- [Demo](docs/demo.md)
- [Evidence Summary](docs/evidence-summary.md)
- [Public Roadmap](docs/public-roadmap.md)
- [Product Brief](docs/product-brief.md)
- [Assessment Offer](docs/assessment-offer.md)
- [Architecture Summary](docs/architecture-summary.md)
- [Threat Model Summary](docs/threat-model-summary.md)
- [Policy Catalog Summary](docs/policy-catalog-summary.md)
- [Signing Lab Summary](docs/signing-lab-summary.md)
- [Observability Summary](docs/observability-summary.md)

## Security

Please read [SECURITY.md](SECURITY.md) before reporting vulnerabilities or using
the project in a lab.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
