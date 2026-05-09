# Public Evidence Pack

This evidence pack summarizes public, sanitized validation for RustShield
Gateway.

Snapshot: 2026-05-07 public showcase hardening.

## Snapshot Metadata

| Field | Value |
|---|---|
| Commit | See publication commit / PR commit |
| Rust toolchain | Rust 1.95.0 in private validation environment |
| `cargo test` | 112 passed |
| `cargo audit` | no vulnerabilities reported |
| `cargo deny check` | advisories, bans, licenses and sources ok |
| Public demo | passed |

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
- [Public Demo Summary](public-demo-summary.md)
- [Claims and Limitations](claims-and-limitations.md)
- [Artifact Hashes](artifact-hashes.md)

## Evidence Context

- [Policy Matrix](../../policy-matrix.md)
- [Test Coverage Summary](../../test-coverage-summary.md)
- [Fixtures Summary](../../fixtures-summary.md)
- [Evidence Ladder](../../evidence-ladder.md)

This public pack is intentionally summarized. It does not include raw private
laboratory logs, private implementation memory, customer material or secrets.
