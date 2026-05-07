# Observability Summary

RustShield Gateway exposes read-only observability suitable for controlled
laboratory evidence capture.

## Surfaces

- structured logs;
- `GET /healthz`;
- `GET /metrics`;
- counters for forwarded, blocked, parse-error and signing-related outcomes;
- latency counters for internal gateway processing;
- shadow-enforcement counters.

## Redaction Boundary

Public observability must not expose:

- MAVLink payloads in full;
- signing keys;
- full signatures;
- private key paths;
- customer or environment secrets.

Observability is diagnostic support. It is not a complete forensic system and
does not imply production monitoring readiness.
