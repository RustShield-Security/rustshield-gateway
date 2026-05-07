# Signing Lab Summary

RustShield Gateway is signing-aware, but public claims are deliberately scoped.

## Modes

- `observe`: observe signed MAVLink 2 packets without claiming authentication.
- `audit`: validate configured signing material in laboratory workflows and log
  validation/rejection results without blocking solely because of signing.
- `enforce`: laboratory path for blocking unsigned, invalid or replayed
  critical/high-risk GCS-to-vehicle commands.

## Claims Boundary

`authenticated=true` is only appropriate after cryptographic validation of
signature, timestamp, configured key policy and link identifier.

Signing support does not replace operational key management, secure enrollment,
rotation, KMS/HSM integration, network segmentation or autopilot hardening.
