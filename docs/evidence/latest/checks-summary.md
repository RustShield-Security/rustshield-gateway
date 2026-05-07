# Checks Summary

Public CI and local validation are expected to run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit
cargo deny check
```

Snapshot result:

| Check | Result |
|---|---|
| `cargo fmt --check` | passed |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed |
| `cargo test` | 112 passed |
| `cargo audit` | no vulnerabilities reported |
| `cargo deny check` | advisories, bans, licenses and sources ok |
| `./scripts/run-public-demo.sh` | passed |

Passing these checks supports technical credibility. It does not prove absence
of vulnerabilities, certification readiness or flight safety.
