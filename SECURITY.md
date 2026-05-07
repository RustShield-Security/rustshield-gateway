# Security Policy

## Scope

This repository is a public technical preview of RustShield Gateway. It is
intended for documentation, simulation and controlled laboratory validation.

In scope for security reports:

- parser crashes or panics on malformed MAVLink input;
- unsafe command forwarding that contradicts the documented public policy;
- leakage of signing keys, payloads or full signatures through logs/metrics;
- dependency vulnerabilities that affect the public build;
- misleading authentication claims in code paths or documentation.

Out of scope:

- real UAV operation;
- physical attacks against autopilots, SD cards, radios or debug ports;
- production deployment hardening not documented in this public preview;
- denial-of-service caused only by deliberately exhausting local lab resources;
- issues in third-party autopilot firmware, GCS software or MAVLink itself.

## Operational Safety

Do not use this project to command real UAV hardware unless there is a separate
controlled safety procedure and explicit authorization. This repository is not
certified for flight operations.

## Reporting

Please report security issues privately by email:

`rustshield.security@proton.me`

Include:

- affected commit or version;
- reproduction steps;
- expected vs actual behavior;
- whether any secret, key, payload or operational evidence may have been exposed.

Please do not open a public GitHub issue for sensitive vulnerabilities.

## Disclosure

The project is early-stage. We will acknowledge reports when possible and
coordinate a fix or documentation clarification before public disclosure.
