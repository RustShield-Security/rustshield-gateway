# Public Claims

## Allowed Claims

- Rust-based MAVLink security gateway technical preview.
- Controlled SITL/laboratory validation.
- Semantic policy checks for selected critical/high-risk MAVLink commands.
- MAVLink signing observability and laboratory enforcement paths.
- Shadow enforcement for non-blocking policy impact assessment.
- Read-only metrics suitable for evidence capture.
- Public evidence summaries for reproducibility review.
- Serial transport validated against virtual PTY devices only.
- Limited PX4 heartbeat handling covered by fixtures and smoke tests.

## Not Allowed Claims

- Formal flight-operations approval.
- Validated for real UAV operation.
- Complete PX4 support.
- Complete MAVLink security coverage.
- Complete Serial/radio hardware support.
- Hard real-time performance guarantee.
- Replacement for platform hardening, key management or network segmentation.
- Combat-grade protection.
- Hijack-proof system.
- Impossible-to-compromise gateway.

## Required Wording

Use:

```text
RustShield Gateway is a Rust-based MAVLink security gateway technical preview
for controlled SITL/laboratory validation.
```

Avoid wording that suggests flight readiness, certification, full autopilot
coverage or hardware validation.
