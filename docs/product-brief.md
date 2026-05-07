# Product Brief

## Problem

MAVLink deployments often need stronger operational controls around command
authorization, signing, observability and evidence. Retrofitting firmware or
changing fleet architecture can be expensive, especially in mixed simulation,
laboratory and integration environments.

## Approach

RustShield Gateway explores an external security gateway that can sit between a
ground control station and an autopilot or simulator. It parses MAVLink traffic,
maintains a conservative state model and evaluates critical commands against a
documented policy.

## Differentiators

- Rust implementation for memory-safety benefits.
- Semantic command filtering instead of raw port filtering.
- MAVLink signing observability and laboratory enforcement experiments.
- Evidence-oriented engineering: ADRs, arc42, threat model, traceability and
  risk register.
- Designed with controlled laboratory validation in mind before hardware claims.

## Current Commercial Position

This is an early technical asset and product prototype, not a certified flight
system. Its current value is in security architecture, policy design, simulation
evidence, and the ability to support controlled lab pilots.

Commercial discussions should start from assessment or laboratory pilot scope,
not production flight deployment.

## Partnership Fit

Potentially relevant for:

- UAV integrators using MAVLink-based systems;
- security laboratories evaluating drone C2 hardening;
- critical infrastructure teams using drones for inspection;
- defense/dual-use R&D groups evaluating command-link protections.

Contact: `rustshield.security@proton.me`
