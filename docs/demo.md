# Public Demo

The public demo is a loopback-only smoke test. It does not require real UAV
hardware, radios, QGroundControl, ArduPilot, PX4 or a GUI.

Run:

```bash
./scripts/run-public-demo.sh
```

The script creates:

```text
target/public-demo-<timestamp>/
  README.md
  gateway.log
  demo.log
  metrics.prom
  expected-results.md
  claims.md
```

## What It Demonstrates

- the gateway starts with a loopback-only configuration;
- a MAVLink critical command can be injected into the GCS-side socket;
- the gateway blocks the command with `CRITICAL-UNKNOWN-001`;
- the gateway records `security.command_blocked`;
- read-only metrics are exposed on loopback;
- `packets_blocked_total 1` is present in `metrics.prom`;
- generated evidence states its limits.

## What It Does Not Demonstrate

- real UAV operation;
- hardware/radio validation;
- QGroundControl UI behavior;
- SITL autopilot behavior;
- flight readiness;
- certification;
- complete MAVLink coverage.
