# Checks Summary

Public CI and local validation are expected to run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit
cargo deny check
```

Passing these checks supports technical credibility. It does not prove absence
of vulnerabilities, certification readiness or flight safety.
