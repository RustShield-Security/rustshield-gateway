#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
config_path="${1:-$repo_root/configs/sitl-gateway.toml}"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
evidence_dir="$repo_root/implementacion/evidencias/sitl-$timestamp"
gateway_log="$evidence_dir/gateway.log"

mkdir -p "$evidence_dir"

{
  echo "# SITL End-to-End Evidence"
  echo
  echo "- Timestamp UTC: $timestamp"
  echo "- Config: $config_path"
  echo "- Gateway log: $gateway_log"
  echo
  echo "## Expected Topology"
  echo
  echo '```text'
  echo "ArduPilot SITL -> udp:127.0.0.1:14560 -> gateway -> udp:127.0.0.1:14552 -> QGroundControl"
  echo "QGroundControl -> udp:127.0.0.1:14551 -> gateway -> udp:127.0.0.1:14540 -> ArduPilot SITL"
  echo '```'
  echo
  echo "## Suggested SITL Command"
  echo
  echo "If ArduCopter SITL is already built:"
  echo
  echo '```bash'
  echo "cd tools/ardupilot"
  echo "build/sitl/bin/arducopter --model + --speedup 1 --slave 0 --sim-address=127.0.0.1 --serial0 udpclient:127.0.0.1:14560 -I0"
  echo '```'
  echo
  echo "If the binary is missing, build it first with the MAVProxy venv on PATH:"
  echo
  echo '```bash'
  echo "cd tools/ardupilot"
  echo "PATH=\"$repo_root/tools/mavproxy-venv/bin:\$PATH\" ./waf configure --board sitl"
  echo "PATH=\"$repo_root/tools/mavproxy-venv/bin:\$PATH\" ./waf build --target bin/arducopter"
  echo '```'
  echo
  echo "## Suggested QGroundControl Link"
  echo
  echo "- Manual UDP link bound to local port 14552."
  echo "- Disable UDP AutoConnect if QGroundControl reports that the bound address is already in use."
  echo "- Vehicle traffic must not connect directly to SITL."
  echo
  echo "## Evidence Checklist"
  echo
  echo "- [ ] Gateway emits \`gateway.start\`."
  echo "- [ ] Gateway emits \`transport.opened\`."
  echo "- [ ] QGroundControl receives telemetry through the gateway."
  echo "- [ ] Gateway emits \`flight_state.mode_changed\` from SITL HEARTBEAT."
  echo "- [ ] Gateway blocks unauthorized critical command with \`security.command_blocked\`."
  echo "- [ ] Gateway emits \`metrics.snapshot\` on shutdown."
  echo
  echo "## Unauthorized Arm Injection"
  echo
  echo "This sends only to the gateway GCS socket in simulation. With"
  echo "\`configs/sitl-gateway.toml\`, loopback \`127.0.0.1\` is intentionally not"
  echo "certified, so an arm attempt should be blocked under \`ARM-AUTO-001\` once"
  echo "SITL is in Auto, or \`CRITICAL-UNKNOWN-001\` before a valid heartbeat."
  echo
  echo '```bash'
  echo "scripts/send-sitl-arm-command.sh 127.0.0.1:14551"
  echo '```'
} > "$evidence_dir/README.md"

echo "Evidence directory: $evidence_dir"
echo "Gateway config: $config_path"
echo "Gateway log: $gateway_log"
echo
echo "Start SITL separately with the command recorded in $evidence_dir/README.md."
echo "Start QGroundControl with a manual UDP link on local port 14552."
echo "Press Ctrl+C here to stop the gateway and write metrics.snapshot."
echo

cd "$repo_root"
cargo run --bin mavlink-shield-gateway -- --config "$config_path" 2>&1 | tee "$gateway_log"
