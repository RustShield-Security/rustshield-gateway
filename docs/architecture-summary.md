# Architecture Summary

RustShield Gateway is designed as an external MAVLink gateway between a Ground
Control Station and an autopilot or simulator.

```text
Ground Control Station <-> RustShield Gateway <-> Autopilot / SITL
```

## Main Components

- MAVLink frame parser and metadata extractor.
- Flight-state tracker based on `HEARTBEAT`.
- Semantic security policy engine.
- MAVLink signing observer/validator for laboratory modes.
- UDP transport for SITL/lab workflows.
- Serial transport boundary validated with virtual PTY devices.
- Read-only metrics endpoint.
- Evidence-generation scripts and public summaries.

## Design Principles

- Treat all MAVLink input as untrusted.
- Preserve MAVLink bytes when forwarding allowed traffic.
- Avoid authentication claims unless signing is cryptographically validated.
- Keep observability read-only by default.
- Prefer loopback/local laboratory defaults.
- Separate technical preview claims from hardware/flight claims.
