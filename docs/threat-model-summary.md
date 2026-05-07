# Threat Model Summary

This public summary describes the threat-model themes addressed by RustShield
Gateway. It is not a complete customer-specific threat model.

## Primary Concerns

- unauthorized critical MAVLink commands;
- unsafe command execution when flight mode is unknown;
- misleading claims around MAVLink signing;
- replay or invalid signed packets in laboratory signing workflows;
- payload, key or full-signature leakage through logs and metrics;
- overly broad network exposure;
- incomplete evidence and unclear operational claims.

## Defensive Controls

- semantic command policy;
- conservative unknown-mode handling;
- signing observe/audit/enforce laboratory paths;
- shadow enforcement for non-blocking impact assessment;
- read-only observability;
- public claims and limitations documentation;
- CI, fuzzing summaries and dependency checks.

## Out of Scope

- real vehicle safety validation;
- physical security;
- firmware vulnerabilities;
- complete MAVLink security coverage;
- customer-specific deployment hardening.
