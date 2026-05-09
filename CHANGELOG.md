# Changelog

## 0.1.0-lab-preview

Prepared documentation for a controlled lab technical preview.

Scope:

- selected high-risk MAVLink command policies;
- MAVLink v1/v2 parsing and byte-preserving forwarding checks;
- signing observe/audit/enforce laboratory paths for selected critical/high-risk
  commands;
- shadow enforcement counters for non-blocking impact assessment;
- virtual PTY Serial transport evidence only;
- limited PX4 heartbeat fixtures with conservative `Unknown` mode handling;
- read-only observability and public evidence summaries.

Checks and public evidence:

- Rust checks: `cargo fmt --check`, `cargo clippy`, `cargo test`;
- supply-chain checks: `cargo audit`, `cargo deny check`;
- public loopback demo: `./scripts/run-public-demo.sh`;
- public evidence context: `docs/evidence-ladder.md`;
- public limitations: `docs/limitations.md`;
- artifact hashes: `docs/evidence/latest/artifact-hashes.md`;
- allowed/not-allowed claims: `docs/claims.md`.

Release note:

- review the exported public checkout manually before tagging, publishing a
  release or pushing a showcase refresh.
- this entry prepares release documentation; it is not proof that a release has
  already been published.

Not claimed:

- real UAV operation;
- hardware/radio validation;
- certification;
- complete MAVLink coverage;
- complete PX4 support;
- hard real-time behavior.

## 0.1.0-public-preview

- Public showcase for a Rust-based MAVLink security gateway.
- Includes core Rust gateway code, architecture documentation, security policy
  documents, threat model and public risk register.
- Adds explicit public-preview safety boundaries.
- Adds CI, license files and security disclosure policy through the showcase
  export workflow.

This public preview is not a flight-operations approval and is not a hardware
validation release.
