# Evidence Ladder

This ladder explains how to interpret RustShield Gateway evidence. Higher
levels provide stronger operational confidence, but future levels are not
claimed by the current public technical preview.

| Level | Public/private | What it demonstrates | What it does not demonstrate | Claims allowed | Claims still prohibited |
|---|---|---|---|---|---|
| Unit tests | Public | Individual parser, policy, signing, metrics and configuration behavior | Full system behavior or external tool compatibility | Source-level checks for selected behaviors | Certification, hardware validation, complete coverage |
| Codec fixtures | Public | MAVLink v1/v2 parsing, selected signed frames, invalid frames and selected command semantics | Every dialect, every message or real telemetry captures | Deterministic fixture coverage | Complete MAVLink protection |
| UDP integration tests | Public | Loopback socket forwarding, byte preservation and selected policy blocks | SITL/GCS UI behavior or network field conditions | Controlled UDP lab behavior | Real network/radio performance claims |
| Signing audit/enforce lab | Public summaries and selected internal evidence | Signed/unsigned/invalid/replay command behavior in controlled lab paths | Production key management, all-message authentication or operational rollout | Laboratory signing validation paths for selected high-risk commands | Complete authentication or operational deployment safety |
| Fuzzing | Public summaries | Parser and policy robustness under generated inputs for time-boxed campaigns | Absence of bugs or exhaustive state coverage | Fuzzing campaign completed without known crashes in stated scope | Formal safety proof |
| Public loopback demo | Public | Local startup, critical command injection, `CRITICAL-UNKNOWN-001`, metrics and shadow counters | SITL, QGroundControl, hardware, radio, flight or certification | Reproducible loopback smoke test | Hardware/radio/flight validation |
| Internal SITL/QGroundControl evidence | Private summary only | Controlled simulator/GCS-oriented behavior in internal lab runs | Public reproducibility of raw logs, hardware or flight | Sanitized SITL-oriented validation summaries | Public raw-evidence claim or field readiness |
| Future physical hardware-lab | Future | Bench behavior with physical hardware under strict safety constraints | Flight, radio field conditions or certification unless separately tested | Not claimed yet | Hardware validated |
| Future radio-lab | Future | Radio-link behavior under controlled lab conditions | Flight operations or full RF assurance | Not claimed yet | Radio validated |
| Future flight validation | Future | Real operational behavior under approved safety process | Formal approval unless separately pursued | Not claimed yet | Operational flight approval, formal assurance, hard real-time guarantees |

## How to Read This

The current public repository sits in the public/source, fixture, loopback and
sanitized-summary part of the ladder. Internal SITL/QGroundControl evidence is
summarized, not published as raw logs.

The public technical preview supports careful evaluation of selected high-risk
MAVLink command policies. It does not support claims of complete protection,
formal approval, real UAV operation, hardware validation, radio validation or
hard real-time behavior.

See [claims.md](claims.md), [limitations.md](limitations.md) and
[demo.md](demo.md).
