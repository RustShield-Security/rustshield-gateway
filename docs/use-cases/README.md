# Use Cases

These use cases are intentionally lab-focused. They describe how RustShield
Gateway can support security review without claiming production or real-flight
readiness.

## 1. Blocked High-Risk MAVLink Command

### Problem

A team wants to understand whether a high-risk MAVLink command can be detected
and blocked before it reaches the vehicle side of a lab or SITL flow.

### Lab Setup

Run RustShield Gateway between a Ground Control Station or command injector and
an ArduPilot SITL-style endpoint using UDP loopback configuration.

### MAVLink Behavior

The gateway observes selected MAVLink command messages such as `COMMAND_LONG`
or `COMMAND_INT` and classifies known high-risk commands against the policy
catalog.

### RustShield Decision

Depending on mode state, source context and policy configuration, the gateway
can block, forward or audit the command. For MVP-style ARM behavior,
`ARM-AUTO-001` blocks non-certified arming attempts in `Automatic` mode, while
unknown mode state is handled conservatively by `CRITICAL-UNKNOWN-001`.

### Evidence Generated

- structured security decision logs;
- counters for received, forwarded, blocked and parse-error packets;
- command-observed and command-blocked metrics;
- policy traceability through the documented rule ID.

### Limitations

- The result is lab evidence, not flight certification.
- IP allowlisting is not strong authentication.
- Coverage is limited to documented command categories and tested scenarios.

## 2. Signing Observe/Audit Mode

### Problem

A team needs to understand how MAVLink signing-related traffic appears in a lab
run without overstating authentication guarantees.

### Lab Setup

Configure a controlled signing observe or audit scenario with known test inputs
and record logs and metrics from the gateway.

### MAVLink Behavior

The gateway can observe MAVLink v2 signing indicators and signing-related
events in supported paths. Observing signed traffic is not the same as proving
that a command is authenticated unless the configured validation path performs
that check.

### RustShield Decision

In observe/audit-oriented modes, the gateway records signing-related facts and
policy decisions without presenting observed signing as automatic trust.

### Evidence Generated

- signing-observed events;
- signing validation or rejection metrics where supported by the configured
  path;
- audit logs for commands that would be blocked under stronger policy;
- limitation notes distinguishing observation from authentication.

### Limitations

- This does not claim complete MAVLink signing coverage.
- Signing behavior must be interpreted within the configured lab path.
- Operational trust decisions require broader validation.

## 3. Shadow Enforcement Impact Assessment

### Problem

A team wants to estimate the operational impact of stricter MAVLink command
policy before enabling blocking behavior.

### Lab Setup

Replay or generate representative lab traffic through the gateway with shadow
enforcement enabled, keeping the environment SITL/lab-only.

### MAVLink Behavior

The gateway evaluates policy outcomes for observed commands while preserving the
configured forwarding behavior where shadow mode is intended to avoid active
blocking.

### RustShield Decision

The gateway reports what policy would have blocked, why, and under which rule,
allowing teams to review potential false positives, expected blocks and missing
policy coverage.

### Evidence Generated

- shadow enforcement findings;
- would-block logs;
- command category summaries;
- metrics snapshots;
- recommendations for policy refinement.

### Limitations

- Shadow results do not prove safe production enforcement.
- Representative traffic quality determines assessment quality.
- Real hardware and operational procedures require separate validation.
