# Proposed GitHub Issues

These issues are small, public and executable. They avoid changing critical
gateway behavior.

## 1. Add a Public Demo Walkthrough

Create a step-by-step markdown walkthrough for the lab demo using existing
scripts and loopback configuration.

Labels: `documentation`, `demo`, `good first issue`

## 2. Add Sample `/healthz` Output

Document an example read-only `/healthz` response with placeholder-safe values
and a note that it is lab-only.

Labels: `documentation`, `observability`

## 3. Add Sample `/metrics` Output

Add a short Prometheus-style metrics sample that shows counters relevant to
packet forwarding, blocking and signing observation.

Labels: `documentation`, `observability`

## 4. Clarify Signing Terminology

Add a glossary section distinguishing signed-observed, validated, rejected,
audit mode and enforce mode.

Labels: `documentation`, `signing`, `claim-boundary`

## 5. Create a Policy Catalog Summary Table

Add a public summary table for critical command categories, rule IDs and current
coverage boundaries.

Labels: `documentation`, `policy-catalog`

## 6. Add a Shadow Enforcement Example

Document a minimal shadow enforcement example showing what a would-block finding
looks like in logs or summary form.

Labels: `documentation`, `security-review`, `evidence`

## 7. Add a Claim Boundary Checklist

Create a checklist for maintainers to review README, releases and demos before
publication.

Labels: `documentation`, `claim-boundary`

## 8. Improve SITL Prerequisites Documentation

Document prerequisites and known environment caveats for running the lab demo
locally.

Labels: `documentation`, `sitl`, `demo`

## 9. Add Evidence Pack Anatomy

Describe the expected contents of a small public evidence pack: commands,
configuration boundary, logs, metrics, limitations and hashes where available.

Labels: `documentation`, `evidence`

## 10. Add a Public Roadmap Boundary Section

Document what future lab work may include and what remains explicitly outside
the public claim until separate validation exists.

Labels: `documentation`, `claim-boundary`
