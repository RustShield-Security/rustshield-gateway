# Policy Catalog Summary

RustShield Gateway includes semantic checks for selected critical/high-risk
MAVLink commands and messages.

Examples include:

- arm/disarm and force-arm;
- unknown-mode critical command blocking;
- navigation movement such as takeoff/land;
- mode changes;
- mission mutation;
- parameter mutation;
- guided reposition;
- RC override/manual control;
- reboot/shutdown;
- sensitive `SETUP_SIGNING` handling.

The catalog is intentionally selective. It should be reviewed and extended for
each autopilot, vehicle type, mission profile and deployment environment.

See [claims.md](claims.md) and [limitations.md](limitations.md) before making
coverage claims.
