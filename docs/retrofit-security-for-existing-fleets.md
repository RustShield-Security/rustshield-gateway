# External Gateway Approach for Existing MAVLink Systems

Many MAVLink fleets cannot safely or economically modify autopilot firmware as
a first step. RustShield explores an external gateway approach to add command
policy evaluation, signing observability and evidence capture before deeper
firmware, platform or operational changes are attempted.

RustShield helps UAV teams introduce MAVLink command-path security controls and
evidence capture around existing systems before committing to deeper firmware or
platform changes.

## Why This Matters

For existing UAV systems, the first security question is often not "how do we
rewrite or replace the autopilot?", but:

- Which MAVLink commands are actually circulating?
- Which commands are high risk in this operational context?
- What would a policy layer block, audit or merely observe?
- What would break if stricter enforcement were enabled?
- What evidence is available for engineering, customer or assurance review?

An external gateway can help answer those questions in a controlled lab/SITL
setting before teams decide whether firmware changes, platform hardening,
network segmentation or formal certification work are justified.

## Operator Problems and Boundaries

| Operator Problem | How RustShield Helps | What It Does Not Solve |
|---|---|---|
| Cannot modify autopilot firmware as a first step | External gateway between GCS and vehicle/simulator | Does not patch internal firmware vulnerabilities |
| Need control over high-risk commands | Semantic MAVLink command policy | Does not certify flight safety |
| Need evidence for customer, internal or regulatory review | Logs, read-only metrics and evidence summaries | Does not replace formal audit or certification |
| Operate or assess a mixed MAVLink environment | Common observation and policy-review layer | Does not guarantee complete PX4/ArduPilot coverage |
| Need to test impact before blocking | Shadow enforcement and would-block findings | Does not validate real UAV operation |

## Appropriate Use

Use this approach for:

- lab/SITL command-path assessment;
- MAVLink policy review;
- signing observability experiments;
- shadow enforcement impact analysis;
- evidence capture for security discussions;
- deciding whether deeper firmware or platform changes are worth pursuing.

## Not a Substitute For

RustShield is not a replacement for:

- autopilot firmware hardening;
- secure key management;
- network segmentation;
- operational safety procedures;
- hardware/radio validation;
- formal certification or regulatory approval.

The project is intentionally conservative: public evidence supports laboratory
validation, not flight readiness.

## Positioning Statement

Before touching firmware, teams need to know what commands circulate, what risks
exist, what policy would block, what operational impact that policy may have and
what evidence can be produced. RustShield provides a lab-oriented layer for
diagnosis, progressive control and evidence around the MAVLink command path.
