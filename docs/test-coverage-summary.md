# Test Coverage Summary

This summary describes what the public RustShield Gateway technical preview
covers and what remains outside the public claim boundary.

It is intentionally high level. It does not publish private raw logs, local
paths, secrets, full payload captures or complete internal implementation
memory.

## Coverage by Area

| Area | Public status | What is covered | What is not covered |
|---|---|---|---|
| MAVLink v1/v2 parsing | Covered by public tests and summaries | Valid MAVLink v1 and v2 frame parsing, routing metadata extraction and byte-preserving forwarding scenarios | Complete dialect coverage and every message family |
| Malformed/truncated/trailing bytes | Covered by tests and fuzzing summaries | Parse errors, truncated frames, checksum failures, bytes after a valid frame and invalid datagrams mapped to safe drop behavior | Exhaustive proof of parser safety |
| Semantic policy | Covered by unit, transport and public policy summaries | Selected high-risk command rules such as arm, unknown-mode blocking, mission mutation, parameter mutation, mode change, takeoff/land, reposition, RC override and reboot/shutdown | Complete command authorization model for every autopilot and mission profile |
| Signing observe/audit/enforce | Covered by tests and lab summaries | Observing signed MAVLink 2 packets, audit validation, command-scope `enforce`, invalid signature, replay and unsigned critical command handling | Full operational key management, all-message authentication or production deployment guidance |
| Replay | Covered by tests and signing lab summaries | Replay rejection in signing validation paths | Distributed clock management and operational replay recovery in production |
| Unexpected `link_id` | Covered by tests and internal audit summary | Rejection of signed packets with unexpected configured `link_id` | Multi-link key rotation or dual-key operations |
| Unsigned critical commands in `enforce` | Covered by tests and public demo-adjacent summaries | Critical/high-risk GCS-to-vehicle commands are blocked when `enforce` requires signing and the packet is unsigned | Non-critical traffic authentication or all-telemetry enforcement |
| `SETUP_SIGNING` | Covered by tests, fuzzing and observability summaries | Sensitive message classification, bidirectional blocking and sanitized metrics/logging | In-gateway key provisioning or operational key management |
| Shadow enforce | Covered by tests, metrics and public demo | Non-blocking `would_block` / `would_reject` counters and events | Runtime enforcement guarantees, fail-safe behavior or replacement for real enforcement |
| Serial virtual PTY | Covered by tests and public summaries | Stream framing, byte preservation, invalid-frame drop, timeout and virtual PTY reconnection | Hardware serial adapters, radios, electrical noise, real UART timing or field use |
| PX4 limited fixtures | Covered by fixtures and smoke tests | PX4 heartbeat parsing with conservative `Unknown` mode handling and command blocking under unknown mode | Full PX4 mode mapping, PX4 SITL campaign, PX4 mission behavior or hardware validation |
| Observability redaction | Covered by tests and public summaries | Read-only health/metrics, sanitized events, no payload/key/full-signature logging in representative checks | Complete forensic logging or formal data-loss-prevention proof |
| Public loopback demo | Covered by script and public evidence | Local gateway startup, critical command injection, `CRITICAL-UNKNOWN-001` block, metrics and shadow counters | SITL, QGroundControl UI, hardware, radio, flight or certification |

## Public Test Boundary

Public checks are intended to support reproducibility review:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo audit
cargo deny check
./scripts/run-public-demo.sh
```

Some checks may need a local environment that allows loopback sockets. The
public demo remains loopback-only.

## Explicit Non-Coverage

- No real UAV operation.
- No hardware/radio validation.
- No certification package.
- No complete MAVLink dialect coverage.
- No guaranteed end-to-end real-time performance.
- No replacement for autopilot hardening, network controls or operational key
  management.

See [policy-matrix.md](policy-matrix.md), [fixtures-summary.md](fixtures-summary.md)
and [evidence-ladder.md](evidence-ladder.md).
