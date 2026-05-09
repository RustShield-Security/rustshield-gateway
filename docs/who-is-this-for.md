# Who Is This For?

RustShield Gateway is for teams that need to understand MAVLink command risk in
a controlled lab or SITL environment. It is not positioned as a flight-approved
component or a production control system.

## UAV Integrators

UAV integrators can use RustShield Gateway to review how high-risk MAVLink
commands move between a Ground Control Station and a vehicle or simulator.

Useful questions:

- Which command paths are visible to the gateway?
- Which commands are treated as critical?
- Which traffic is forwarded, blocked or audited?
- What evidence is available for a safety or security review?

Boundary:

- The project does not replace autopilot hardening, secure firmware settings,
  link-layer security or operational procedures.
- Real aircraft validation is outside the current public claim.

## Drone Security Labs

Security labs can use the gateway as a repeatable test target for MAVLink
parsing, policy behavior, signing observability and evidence capture.

Useful questions:

- Can a high-risk command be identified before it reaches the vehicle side?
- How are malformed or unsupported frames handled?
- What does signing observe/audit/enforce behavior show in a lab run?
- Which residual risks remain after policy evaluation?

Boundary:

- The project is not a comprehensive MAVLink security solution.
- Results should be reported as lab findings unless separately validated in a
  broader environment.

## Critical Infrastructure Inspection Teams

Teams that inspect power, transport, water, telecommunications or other critical
sites can use the project to explore command-policy concepts before adopting or
requesting stronger UAV security controls.

Useful questions:

- Which commands would be considered high risk during sensitive operations?
- What logs and metrics would an assurance review need?
- How could command policy decisions be discussed with integrators and
  operators?

Boundary:

- This repository does not authorize operational use near critical
  infrastructure.
- Operational deployment requires procedures, environment validation and
  independent safety/security review.

## Defense / Dual-Use R&D Groups

R&D teams can use RustShield Gateway as a transparent lab asset for studying
semantic command policy, MAVLink signing visibility and evidence generation.

Useful questions:

- What does a policy layer need to know before allowing a command?
- How should unsigned, invalidly signed or unexpected-link traffic be reported?
- What traceability is needed before moving from lab evidence to field trials?

Boundary:

- This repository does not claim military qualification, certification or field
  readiness.
- Any dual-use evaluation should remain within authorized, controlled and
  legally compliant lab environments.

## Academic Robotics and Security Labs

Academic teams can use the repository to teach and study MAVLink security
concepts, Rust implementation patterns, threat modeling and evidence-oriented
engineering.

Useful questions:

- How can a protocol gateway separate parsing, state tracking, policy and
  observability?
- How can fuzzing and tests support parser and policy confidence?
- How should limitations be documented when evidence is incomplete?

Boundary:

- Publications or demonstrations should preserve the project limitations and
  avoid presenting lab results as flight validation.
