# Evidence Summary

This public summary describes the type of evidence maintained by the project.
It is not a certification package.

## Publicly Representable Evidence

- Rust formatting, linting and test execution.
- Dependency and license checks using `cargo audit` and `cargo deny`.
- MAVLink parser fixtures and policy tests.
- Simulation-oriented SITL procedures.
- Architecture documentation, ADRs, traceability and risk register.
- Public risk statements describing unresolved hardware, PX4, signing and
  operational limitations.

## Evidence Kept Private Unless Cleared

- raw internal evidence packs;
- environment-specific logs;
- full laboratory timelines;
- detailed private roadmap notes;
- any artifact containing local paths, operator workflow details or sensitive
  integration context.

## Recommended Public CI Checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit
cargo deny check
```

## Interpretation

Passing these checks supports technical credibility. It does not prove absence
of vulnerabilities, compliance with aviation standards or safety for real
flight operations.
