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
- `shadow_policy_would_block_total 1` is present in `metrics.prom`;
- `shadow_signing_would_reject_total 1` is present in `metrics.prom`;
- `commands_critical_observed_total 1` is present in `metrics.prom`;
- generated evidence states its limits.

## Why `CRITICAL-UNKNOWN-001`

The public demo intentionally avoids SITL, QGroundControl and real hardware.
Because no simulator heartbeat is required, the gateway has no trusted flight
mode context and classifies the mode as `Unknown`.

In that state, the conservative behavior is to block selected critical/high-risk
commands with `CRITICAL-UNKNOWN-001`. This makes the demo deterministic and safe
for a public loopback smoke test.

## What The Script Checks

`./scripts/run-public-demo.sh` starts the gateway with loopback-only bindings,
injects one selected critical command, captures sanitized evidence and fails if
the expected block or metrics are missing.

The deterministic checks are:

- `security.command_blocked` appears in `gateway.log`;
- `CRITICAL-UNKNOWN-001` appears in `gateway.log`;
- `packets_blocked_total 1` appears in `metrics.prom`;
- `shadow_policy_would_block_total 1` appears in `metrics.prom`;
- `shadow_signing_would_reject_total 1` appears in `metrics.prom`.

## Relationship To Shadow Enforce

Shadow enforce records what stricter policy or signing enforcement would have
rejected without making shadow mode the reason for the block. In this demo, the
real block is the semantic `CRITICAL-UNKNOWN-001` decision. The shadow counters
provide additional impact evidence for the same command path.

## What It Does Not Demonstrate

- real UAV operation;
- hardware/radio validation;
- QGroundControl UI behavior;
- SITL autopilot behavior;
- flight readiness;
- certification;
- complete MAVLink coverage.

See [policy-matrix.md](policy-matrix.md),
[test-coverage-summary.md](test-coverage-summary.md) and
[evidence-ladder.md](evidence-ladder.md).
