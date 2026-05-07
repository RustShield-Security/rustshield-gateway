# Contributing

RustShield Gateway is security-sensitive. Contributions should preserve the
public-preview boundaries and avoid expanding operational claims without
evidence.

## Development Checks

Run before proposing changes:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

Dependency/security checks used by the project:

```bash
cargo audit
cargo deny check
```

## Safety Rules

- Do not add real UAV operation procedures to this public preview.
- Do not add arming, takeoff, landing, mission upload, parameter mutation or RC
  override workflows for real hardware.
- Do not log MAVLink payloads, signing keys, full signatures or operational
  secrets.
- Do not claim certification, production readiness or hardware validation unless
  an evidence artifact exists and is reviewed.

## Documentation Rules

Security, compatibility, latency or operational decisions should be documented
through ADRs or explicit public-scope notes.

Claims should be phrased as evidence-backed statements, not marketing promises.
