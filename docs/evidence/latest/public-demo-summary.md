# Public Demo Summary

Snapshot: 2026-05-07 public demo smoke test.

Expected deterministic result:

- `security.command_blocked` appears in `gateway.log`;
- `CRITICAL-UNKNOWN-001` appears in `gateway.log`;
- `packets_blocked_total 1` appears in `metrics.prom`;
- `shadow_policy_would_block_total 1` appears in `metrics.prom`;
- `shadow_signing_would_reject_total 1` appears in `metrics.prom`.

The demo is loopback-only. It does not validate real UAV hardware, radio links,
QGroundControl UI behavior, ArduPilot/PX4 SITL behavior, flight readiness or
certification.
