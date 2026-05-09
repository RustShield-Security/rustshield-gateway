# Public Demo Summary

Snapshot: 2026-05-07 public demo smoke test.

Expected deterministic result:

- `security.command_blocked` appears in `gateway.log`;
- `CRITICAL-UNKNOWN-001` appears in `gateway.log`;
- `packets_blocked_total 1` appears in `metrics.prom`;
- `shadow_policy_would_block_total 1` appears in `metrics.prom`;
- `shadow_signing_would_reject_total 1` appears in `metrics.prom`.
- `commands_critical_observed_total 1` appears in `metrics.prom`.

Why this case uses `CRITICAL-UNKNOWN-001`:

- the public demo is intentionally loopback-only;
- it does not require a simulator heartbeat;
- without trusted flight-mode context, the gateway treats the mode as
  `Unknown`;
- selected critical/high-risk commands are blocked in that state.

The shadow counters show non-blocking impact evidence for stricter policy and
signing behavior. They do not replace the real block reason in this demo.

The demo is loopback-only. It does not validate real UAV hardware, radio links,
QGroundControl UI behavior, ArduPilot/PX4 SITL behavior, flight readiness or
certification.
