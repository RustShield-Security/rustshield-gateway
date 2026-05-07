# Limitations

RustShield Gateway is a public technical preview. Its current public evidence is
limited by design.

## Technical Limits

- MAVLink coverage is focused on selected messages and commands from the common
  dialect.
- Policy coverage is intentionally selective and centered on critical/high-risk
  command classes.
- ArduPilot Copter SITL is the primary documented workflow.
- PX4 handling is limited and conservative; PX4 modes are not fully mapped.
- Serial transport evidence is virtual PTY evidence, not hardware/radio
  validation.
- MAVLink signing support is laboratory-oriented and does not replace key
  management.
- Read-only observability does not provide a complete forensic record.

## Operational Limits

- No real UAV operation is validated by this public repository.
- No flight safety claim is made.
- No certification claim is made.
- No hardware/radio claim is made.
- No guaranteed end-to-end real-time performance is claimed.

## Evidence Limits

Public evidence is sanitized and summarized. Raw internal logs, private
laboratory history and environment-specific details are not published unless
explicitly cleared.
