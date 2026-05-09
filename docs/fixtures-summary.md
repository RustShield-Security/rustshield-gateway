# Fixtures Summary

RustShield Gateway uses synthetic fixtures and generated MAVLink frames to
exercise parser, policy, signing and transport behavior in controlled
laboratory-oriented tests.

The public repository does not publish hardware-derived captures unless they
are explicitly reviewed and cleared.

## Fixture Types

| Fixture type | Public purpose | Notes |
|---|---|---|
| Synthetic fixtures | Deterministic parser and policy tests | Generated in controlled tests rather than captured from a field system |
| MAVLink v1 | Validate v1 framing, routing metadata and transparent forwarding | Used for parser and transport compatibility checks |
| MAVLink v2 | Validate v2 framing, routing metadata and transparent forwarding | Includes unsigned v2 traffic |
| MAVLink v2 signed | Validate signing observation and lab validation paths | Public summaries avoid publishing secrets, key material or full signatures |
| Malformed/truncated | Validate safe parse errors and invalid-frame drops | Includes truncated frames, checksum alteration and trailing bytes |
| Critical/high-risk commands | Validate selected policy rules | Includes arm, takeoff/land, mode change, mission mutation, parameter mutation, reposition, RC override and reboot/shutdown classes |
| `SETUP_SIGNING` | Validate sensitive-message handling | Blocked and summarized without payload disclosure |
| PX4 limited | Validate conservative PX4 heartbeat behavior | PX4 modes are treated as `Unknown` in the limited public scope |

## What Is Not Published

- Private laboratory raw logs.
- Full internal implementation memory.
- Local filesystem paths from private workstations.
- Secrets, signing keys or key file paths.
- Full MAVLink payload captures when not needed for public reproducibility.
- Complete signing signatures.
- Customer or partner material.
- Hardware-derived fixtures unless explicitly cleared later.

## Fixture Claim Boundary

Fixtures support controlled repeatability. They do not demonstrate physical
hardware behavior, radio behavior, real flight behavior, complete autopilot
coverage or production readiness.

See [test-coverage-summary.md](test-coverage-summary.md) and
[evidence-ladder.md](evidence-ladder.md).
