# Public Evidence Pack

This evidence pack summarizes public, sanitized validation for RustShield
Gateway.

## Scope

- Controlled SITL/laboratory workflows.
- Local Rust checks.
- Public documentation and claim boundaries.
- No real UAV operation.
- No hardware/radio validation.
- No certification claim.

## Checks

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo audit`
- `cargo deny check`

## Evidence Summaries

- [Checks Summary](checks-summary.md)
- [SITL Summary](sitl-summary.md)
- [Signing Summary](signing-summary.md)
- [Fuzz Summary](fuzz-summary.md)
- [Latency Summary](latency-summary.md)
- [Claims and Limitations](claims-and-limitations.md)
- [Artifact Hashes](artifact-hashes.md)

This public pack is intentionally summarized. It does not include raw private
laboratory logs, private implementation memory, customer material or secrets.
