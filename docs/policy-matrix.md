# Policy Matrix

This matrix summarizes selected high-risk MAVLink policies covered by the
RustShield Gateway public technical preview.

It is a public evidence summary, not a full MAVLink security specification. The
catalog focuses on selected command classes that are useful for controlled
SITL/laboratory validation.

## Matrix

| Rule | MAVLink traffic | Severity | Direction | Condition | Action | Signing enforce impact | Public coverage |
|---|---|---|---|---|---|---|---|
| `ARM-AUTO-001` | `COMMAND_LONG` / `COMMAND_INT` with `MAV_CMD_COMPONENT_ARM_DISARM` | Critical | GCS to vehicle | Arm request while flight mode is classified as `Automatic` and source is not approved by policy | Block | In `enforce`, unsigned, invalid or replayed critical arm commands are rejected before semantic allow | Unit and UDP tests; internal SITL evidence summaries |
| `CRITICAL-UNKNOWN-001` | Cataloged critical/high-risk commands | Critical | GCS to vehicle | Flight mode is `Unknown` | Block | In `enforce`, unsigned, invalid or replayed critical commands are rejected before semantic policy | Public loopback demo and unit/UDP tests |
| `PARAM-SET-001` | `PARAM_SET` | High | GCS to vehicle | Parameter mutation from a source not approved by policy when mode is known | Block | Treated as high-risk command traffic under signing enforcement scope | Unit and codec coverage summaries |
| `MISSION-UPLOAD-001` | `MISSION_COUNT`, `MISSION_ITEM`, `MISSION_ITEM_INT`, `MISSION_CLEAR_ALL`, `MISSION_SET_CURRENT`, `MISSION_WRITE_PARTIAL_LIST` | High | GCS to vehicle | Mission mutation from a source not approved by policy when mode is known | Block | Treated as high-risk command traffic under signing enforcement scope | Unit and codec coverage summaries |
| `MODE-CHANGE-001` | `SET_MODE`, `MAV_CMD_DO_SET_MODE` | High | GCS to vehicle | Mode change request from a source not approved by policy when mode is known | Block | Treated as high-risk command traffic under signing enforcement scope | Unit and codec coverage summaries |
| `NAV-MOVEMENT-001` | `MAV_CMD_NAV_TAKEOFF`, `MAV_CMD_NAV_LAND` | Critical | GCS to vehicle | Takeoff or landing command from a source not approved by policy when mode is known | Block | Unsigned takeoff/land commands are blocked in `enforce` before semantic policy | Unit, codec and transport coverage summaries |
| `GUIDED-REPOSITION-001` | `MAV_CMD_DO_REPOSITION` | High | GCS to vehicle | Guided reposition request from a source not approved by policy when mode is known | Block | Treated as high-risk command traffic under signing enforcement scope | Unit and fuzz coverage summaries |
| `RC-OVERRIDE-001` | `MANUAL_CONTROL`, `RC_CHANNELS_OVERRIDE` | High | GCS to vehicle | Manual control or RC override from a source not approved by policy when mode is known | Block | Treated as high-risk command traffic under signing enforcement scope | Unit, codec and fuzz coverage summaries |
| `PREFLIGHT-REBOOT-001` | `MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN` | Critical | GCS to vehicle | Reboot or shutdown command from a source not approved by policy when mode is known | Block | Treated as critical command traffic under signing enforcement scope | Unit and codec coverage summaries |
| `PARSE-ERROR-001` | Malformed, truncated or unsupported MAVLink datagram | Medium | Both directions | Datagram cannot be parsed as a valid single MAVLink frame | Drop invalid datagram | Signing validation is not reached for invalid frames | Unit, fuzz and metrics coverage summaries |
| `SETUP_SIGNING` | MAVLink `SETUP_SIGNING` message | Sensitive | Both directions | `SETUP_SIGNING` observed on gateway path | Block and record sanitized observation | Not used to provision signing keys through the gateway; no payload is logged | Unit, transport, fuzz and observability coverage summaries |

## Public Claims This Supports

- Selected high-risk command policies are implemented and covered by public
  summaries.
- The public demo verifies `CRITICAL-UNKNOWN-001` in a loopback-only scenario.
- Signing `enforce` is scoped to critical/high-risk command traffic in
  controlled laboratory validation paths.
- Sensitive `SETUP_SIGNING` traffic is blocked and summarized without exposing
  payload or key material.

## Limits

- This matrix is not comprehensive MAVLink protection.
- It does not cover every MAVLink dialect, autopilot, vehicle class or mission
  profile.
- Public coverage is a combination of source tests, sanitized summaries and the
  loopback demo; some SITL/QGroundControl evidence remains private.
- Hardware, radio and flight validation are future work.

See [claims.md](claims.md), [limitations.md](limitations.md) and
[evidence-ladder.md](evidence-ladder.md).
