#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
evidence_dir="${1:-$repo_root/target/public-demo-$timestamp}"
config_path="$evidence_dir/public-demo.toml"
gateway_log="$evidence_dir/gateway.log"
demo_log="$evidence_dir/demo.log"
metrics_path="$evidence_dir/metrics.prom"

mkdir -p "$evidence_dir"

cat > "$config_path" <<'CONFIG'
[transport]
mode = "udp"

[udp]
listen_vehicle = "127.0.0.1:14650"
listen_gcs = "127.0.0.1:14651"
vehicle_addr = "127.0.0.1:14652"
gcs_addr = "127.0.0.1:14653"
read_timeout_ms = 100
max_datagram_size = 2048

[security]
certified_ips = ["127.0.0.2"]
unknown_mode_policy = "block"
audit_only = false
shadow_enforce = true
block_arm_in_auto_mode = true

[signing]
policy = "observe"
link_id = 0

[crypto]
enabled = false

[logging]
level = "info"
payload_logging = false

[metrics]
enabled = true
readonly_bind = "127.0.0.1:14660"
CONFIG

cleanup() {
  if [[ -n "${gateway_pid:-}" ]] && kill -0 "$gateway_pid" >/dev/null 2>&1; then
    kill -INT "$gateway_pid" >/dev/null 2>&1 || true
    wait "$gateway_pid" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

cd "$repo_root"

cargo build --bins >>"$demo_log" 2>&1

target/debug/mavlink-shield-gateway --config "$config_path" >"$gateway_log" 2>&1 &
gateway_pid="$!"

gateway_ready=false
for _ in $(seq 1 30); do
  if grep -q 'transport.opened' "$gateway_log" 2>/dev/null; then
    gateway_ready=true
    break
  fi
  sleep 0.2
done

if [[ "$gateway_ready" != true ]]; then
  echo "public demo failed: gateway did not report transport.opened" >&2
  exit 1
fi

target/debug/sitl-send-arm-command 127.0.0.1:14651 >>"$demo_log" 2>&1
sleep 0.5

if command -v curl >/dev/null 2>&1; then
  curl -fsS "http://127.0.0.1:14660/metrics" > "$metrics_path"
else
  printf 'curl not available; metrics not captured\n' > "$metrics_path"
fi

cleanup
trap - EXIT

if ! grep -q 'event="security.command_blocked"' "$gateway_log"; then
  echo "public demo failed: security.command_blocked event not found" >&2
  exit 1
fi

if ! grep -q 'rule_id="CRITICAL-UNKNOWN-001"' "$gateway_log"; then
  echo "public demo failed: CRITICAL-UNKNOWN-001 not found" >&2
  exit 1
fi

if ! grep -q '^packets_blocked_total 1$' "$metrics_path"; then
  echo "public demo failed: expected packets_blocked_total 1" >&2
  exit 1
fi

if ! grep -q '^shadow_policy_would_block_total 1$' "$metrics_path"; then
  echo "public demo failed: expected shadow_policy_would_block_total 1" >&2
  exit 1
fi

if ! grep -q '^shadow_signing_would_reject_total 1$' "$metrics_path"; then
  echo "public demo failed: expected shadow_signing_would_reject_total 1" >&2
  exit 1
fi

cat > "$evidence_dir/expected-results.md" <<'EOF'
# Expected Results

- Gateway starts on loopback-only UDP sockets.
- A MAVLink `MAV_CMD_COMPONENT_ARM_DISARM` attempt is sent to the GCS-side
  gateway socket.
- Because no vehicle heartbeat established a supported safe mode and the
  source IP is not certified, the command must be blocked by
  `CRITICAL-UNKNOWN-001`.
- `gateway.log` must contain `security.command_blocked`.
- `metrics.prom` must contain `packets_blocked_total 1`.
- `metrics.prom` must contain `shadow_policy_would_block_total 1`.
- `metrics.prom` must contain `shadow_signing_would_reject_total 1`.
EOF

cat > "$evidence_dir/claims.md" <<'EOF'
# Demo Claims

## Allowed

- Loopback-only public smoke demo.
- No real UAV hardware.
- No radio.
- No flight operation.
- Read-only metrics capture.

## Not Allowed

- Flight readiness.
- Certification.
- Hardware/radio validation.
- Complete MAVLink security coverage.
EOF

cat > "$evidence_dir/README.md" <<EOF
# Public Demo Evidence

- Timestamp UTC: $timestamp
- Config: public-demo.toml
- Gateway log: gateway.log
- Demo log: demo.log
- Metrics: metrics.prom

This evidence is a loopback-only smoke test. It does not validate real UAV
hardware, radio links, flight behavior, QGroundControl UI behavior or
certification readiness.
EOF

printf '%s\n' "$evidence_dir"
