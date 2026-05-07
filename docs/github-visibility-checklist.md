# GitHub Visibility Checklist

This checklist proposes small public-repository improvements that do not change
the core gateway logic.

## Suggested Repo Topics

- `mavlink`
- `uav-security`
- `drone-security`
- `rust`
- `sitl`
- `ardupilot`
- `qgroundcontrol`
- `security-gateway`
- `cyber-physical-systems`
- `robotics-security`
- `threat-modeling`
- `observability`

## Release Naming

Use conservative lab-oriented release names:

- `v0.1.0-lab-preview`
- `v0.1.1-public-demo-refresh`
- `v0.2.0-signing-observability-lab`
- `v0.3.0-shadow-enforcement-lab`

Avoid release names that imply flight readiness, certification or production
deployment.

## Demo GIF / Video Recommendation

Create a short screen recording or GIF showing:

1. gateway startup with lab configuration;
2. a MAVLink command entering the gateway;
3. a policy decision in logs;
4. `/healthz` or `/metrics` read-only output;
5. a short evidence summary.

The caption should say "SITL/lab demo" and should not show real UAV operation.

## Discussion Topics

- MAVLink command policy design in lab environments.
- How to document evidence boundaries for UAV security tools.
- Signing observability versus authentication claims.
- Shadow enforcement and false-positive review.
- SITL validation patterns for security-sensitive robotics projects.

## Issue Labels

- `documentation`
- `good first issue`
- `demo`
- `observability`
- `policy-catalog`
- `signing`
- `sitl`
- `security-review`
- `evidence`
- `claim-boundary`

## First Public Issues

Start with issues that improve clarity and evidence quality:

- README clarity and claim-boundary review.
- Public demo walkthrough.
- Use-case documentation.
- Metrics sample output.
- Signing terminology clarification.
- Policy catalog summary.
- Evidence pack example.
- Contributor guide for lab-only scope.
