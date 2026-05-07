# Commercial Pilot Package

RustShield Gateway can support a small laboratory assessment package for teams
that want to understand MAVLink command risk before considering operational
security work.

This is a lab/SITL offer. It is not a production deployment, a flight validation
campaign or a certification package.

## Suggested Duration

2-4 weeks.

The exact duration depends on access to representative MAVLink traffic,
available SITL setup, policy-review scope and reporting depth.

## Environment

- SITL or lab-only environment.
- No real UAV flight operation.
- No authorization to arm, take off, land, change flight mode, upload missions,
  mutate parameters or use RC override on real hardware.
- Traffic capture, replay or simulation should be approved by the customer and
  documented before work starts.

## Typical Inputs

- MAVLink traffic samples or SITL scenario description.
- Target autopilot and GCS assumptions.
- Known high-risk commands or operating constraints.
- Existing security procedures or policy expectations.
- Current signing or telemetry observability requirements.

## Deliverables

### MAVLink Traffic Assessment

Summary of observed MAVLink message families, command paths, parsing behavior
and unsupported or ambiguous traffic seen during the lab work.

### Command-Policy Review

Review of high-risk command categories against the documented policy catalog,
including what is covered, what is only observed and what remains out of scope.

### Signing Observability Report

Report on signing-related observations in the lab environment, clearly
distinguishing observed signing metadata from validated authentication.

### Shadow Enforcement Findings

Summary of what the gateway would have blocked or audited under configured
policy modes, with enough context to review operational impact before any
stronger enforcement decision.

### Evidence Summary

Short evidence pack with commands run, configuration boundary, relevant logs,
metrics snapshots and known limitations.

### Risk Register

Lab-specific risk register covering command authorization, observability gaps,
unsupported modes, signing limitations, replay assumptions and operational
unknowns.

### Recommendations

Practical recommendations for next steps, such as additional SITL cases, policy
refinement, autopilot hardening review, signing validation work or a separate
hardware-lab plan.

## Not Included

- Certified flight safety.
- Real aircraft validation.
- Production deployment.
- Autopilot firmware hardening.
- Complete MAVLink security coverage.
- Generic HTTP/API gateway work.
- Certification claims or compliance attestation.
