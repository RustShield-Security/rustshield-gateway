#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
gateway_log="${1:?usage: scripts/collect-latency-evidence.sh <gateway.log> [evidence-dir]}"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"
evidence_dir="${2:-$repo_root/implementacion/evidencias/latency-e2e-sitl-$timestamp}"

if [[ ! -f "$gateway_log" ]]; then
  echo "gateway log not found: $gateway_log" >&2
  exit 1
fi

snapshot="$(grep 'event="metrics.snapshot"' "$gateway_log" | tail -n 1 || true)"
if [[ -z "$snapshot" ]]; then
  echo "metrics.snapshot not found in $gateway_log" >&2
  exit 1
fi

mkdir -p "$evidence_dir"
cp "$gateway_log" "$evidence_dir/gateway.log"

metric() {
  local name="$1"
  local value
  value="$(printf '%s\n' "$snapshot" | grep -o "${name}=[^ ]*" | tail -n 1 | cut -d= -f2- || true)"
  printf '%s' "${value:-0}"
}

avg_us() {
  local total="$1"
  local samples="$2"
  awk -v total="$total" -v samples="$samples" 'BEGIN {
    if (samples == 0) {
      printf "n/a";
    } else {
      printf "%.3f", total / samples;
    }
  }'
}

packets_received_total="$(metric packets_received_total)"
packets_forwarded_total="$(metric packets_forwarded_total)"
packets_blocked_total="$(metric packets_blocked_total)"
packets_parse_error_total="$(metric packets_parse_error_total)"
packets_signed_observed_total="$(metric packets_signed_observed_total)"
commands_critical_observed_total="$(metric commands_critical_observed_total)"
processing_latency_samples="$(metric processing_latency_samples)"
processing_latency_total_us="$(metric processing_latency_total_us)"
processing_latency_max_us="$(metric processing_latency_max_us)"
parse_latency_samples="$(metric parse_latency_samples)"
parse_latency_total_us="$(metric parse_latency_total_us)"
parse_latency_max_us="$(metric parse_latency_max_us)"
policy_latency_samples="$(metric policy_latency_samples)"
policy_latency_total_us="$(metric policy_latency_total_us)"
policy_latency_max_us="$(metric policy_latency_max_us)"

processing_avg_us="$(avg_us "$processing_latency_total_us" "$processing_latency_samples")"
parse_avg_us="$(avg_us "$parse_latency_total_us" "$parse_latency_samples")"
policy_avg_us="$(avg_us "$policy_latency_total_us" "$policy_latency_samples")"

{
  echo "# Latency End-to-End SITL Evidence"
  echo
  echo "- Timestamp UTC: $timestamp"
  echo "- Source log: $gateway_log"
  echo "- Copied log: gateway.log"
  echo "- Scope: UDP local + gateway + ArduPilot Copter SITL/QGroundControl evidence"
  echo
  echo "## Measurement Boundary"
  echo
  echo "This evidence extracts the gateway internal latency counters from"
  echo "\`metrics.snapshot\`. It does not claim hard real-time behavior and does not"
  echo "measure QGroundControl UI latency, OS scheduling, network latency, or simulator"
  echo "control-loop latency."
  echo
  echo "## Metrics Snapshot"
  echo
  echo "| Metric | Value |"
  echo "|---|---:|"
  echo "| packets_received_total | $packets_received_total |"
  echo "| packets_forwarded_total | $packets_forwarded_total |"
  echo "| packets_blocked_total | $packets_blocked_total |"
  echo "| packets_parse_error_total | $packets_parse_error_total |"
  echo "| packets_signed_observed_total | $packets_signed_observed_total |"
  echo "| commands_critical_observed_total | $commands_critical_observed_total |"
  echo "| processing_latency_samples | $processing_latency_samples |"
  echo "| processing_latency_total_us | $processing_latency_total_us |"
  echo "| processing_latency_max_us | $processing_latency_max_us |"
  echo "| parse_latency_samples | $parse_latency_samples |"
  echo "| parse_latency_total_us | $parse_latency_total_us |"
  echo "| parse_latency_max_us | $parse_latency_max_us |"
  echo "| policy_latency_samples | $policy_latency_samples |"
  echo "| policy_latency_total_us | $policy_latency_total_us |"
  echo "| policy_latency_max_us | $policy_latency_max_us |"
  echo
  echo "## Derived Internal Latency"
  echo
  echo "| Metric | Value |"
  echo "|---|---:|"
  echo "| processing_avg_us | $processing_avg_us |"
  echo "| parse_avg_us | $parse_avg_us |"
  echo "| policy_avg_us | $policy_avg_us |"
  echo
  echo "## Interpretation"
  echo
  echo "- Gateway internal processing max is documented from the final snapshot."
  echo "- Forwarding throughput is represented by forwarded packet counters, not by a"
  echo "  wall-clock packet rate unless the run duration is documented separately."
  echo "- Product claims must remain scoped to internal gateway processing unless a"
  echo "  future harness adds correlated timestamps at sender, gateway and receiver."
  if [[ "$processing_latency_samples" != "$packets_received_total" ]]; then
    echo "- \`processing_latency_samples\` differs from \`packets_received_total\`;"
    echo "  inspect the gateway version and traffic mix before treating averages as"
    echo "  complete per-packet processing coverage."
  fi
  echo
  echo "## Raw Snapshot"
  echo
  echo '```text'
  echo "$snapshot"
  echo '```'
} > "$evidence_dir/README.md"

echo "$evidence_dir"
