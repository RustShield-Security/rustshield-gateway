# Market Positioning

RustShield Gateway is best described as a MAVLink security validation gateway
for controlled lab and SITL workflows.

## RustShield Gateway Is

- a MAVLink security validation gateway;
- a semantic command policy layer;
- a lab/SITL evidence generator;
- a security assessment asset for UAV systems.

## RustShield Gateway Is Not

- a certified flight system;
- a replacement for autopilot hardening;
- a generic HTTP gateway;
- a generic NGINX/Envoy competitor;
- a complete MAVLink security solution;
- validated for real UAV flight.

## Positioning Statement

RustShield Gateway helps UAV and security teams examine high-risk MAVLink
command behavior in a controlled lab environment. It is useful when the question
is not "can this fly in production today?", but rather "what command risk can
we observe, explain, test and document before stronger assurance work begins?"

## Messaging Guardrails

Use:

- "technical preview";
- "lab/SITL validation";
- "security assessment asset";
- "semantic command policy";
- "evidence-oriented review";
- "read-only observability for lab review".

Avoid:

- "flight-ready";
- "certified";
- "production-safe";
- "military-grade";
- "complete protection";
- "drop-in aircraft security";
- "guaranteed real-time protection".

## Evidence Boundary

Claims should stay aligned with repository evidence:

- unit and integration tests support parser and policy behavior;
- SITL/lab procedures support controlled demonstration;
- metrics and logs support reviewability;
- documented limitations remain part of the public value proposition.
