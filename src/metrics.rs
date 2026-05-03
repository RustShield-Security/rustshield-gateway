#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GatewayCounters {
    pub packets_received_total: u64,
    pub packets_forwarded_total: u64,
    pub packets_blocked_total: u64,
    pub packets_parse_error_total: u64,
    pub packets_signed_observed_total: u64,
    pub packets_signed_valid_total: u64,
    pub packets_signed_invalid_total: u64,
    pub packets_unsigned_rejected_total: u64,
    pub signing_replay_rejected_total: u64,
    pub setup_signing_observed_total: u64,
    pub commands_critical_observed_total: u64,
    pub processing_latency_samples: u64,
    pub processing_latency_total_us: u64,
    pub processing_latency_max_us: u64,
    pub parse_latency_samples: u64,
    pub parse_latency_total_us: u64,
    pub parse_latency_max_us: u64,
    pub policy_latency_samples: u64,
    pub policy_latency_total_us: u64,
    pub policy_latency_max_us: u64,
    pub last_heartbeat_age_ms: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricName {
    PacketsReceivedTotal,
    PacketsForwardedTotal,
    PacketsBlockedTotal,
    PacketsParseErrorTotal,
    PacketsSignedObservedTotal,
    PacketsSignedValidTotal,
    PacketsSignedInvalidTotal,
    PacketsUnsignedRejectedTotal,
    SigningReplayRejectedTotal,
    SetupSigningObservedTotal,
    CommandsCriticalObservedTotal,
    ProcessingLatencyUs,
    ParseLatencyUs,
    PolicyLatencyUs,
    LastHeartbeatAgeMs,
}

impl MetricName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PacketsReceivedTotal => "packets_received_total",
            Self::PacketsForwardedTotal => "packets_forwarded_total",
            Self::PacketsBlockedTotal => "packets_blocked_total",
            Self::PacketsParseErrorTotal => "packets_parse_error_total",
            Self::PacketsSignedObservedTotal => "packets_signed_observed_total",
            Self::PacketsSignedValidTotal => "packets_signed_valid_total",
            Self::PacketsSignedInvalidTotal => "packets_signed_invalid_total",
            Self::PacketsUnsignedRejectedTotal => "packets_unsigned_rejected_total",
            Self::SigningReplayRejectedTotal => "signing_replay_rejected_total",
            Self::SetupSigningObservedTotal => "setup_signing_observed_total",
            Self::CommandsCriticalObservedTotal => "commands_critical_observed_total",
            Self::ProcessingLatencyUs => "processing_latency_us",
            Self::ParseLatencyUs => "parse_latency_us",
            Self::PolicyLatencyUs => "policy_latency_us",
            Self::LastHeartbeatAgeMs => "last_heartbeat_age_ms",
        }
    }
}

impl GatewayCounters {
    pub fn record_received_packet(&mut self) {
        self.packets_received_total += 1;
    }

    pub fn record_forwarded_packet(&mut self) {
        self.packets_forwarded_total += 1;
    }

    pub fn record_blocked_packet(&mut self) {
        self.packets_blocked_total += 1;
    }

    pub fn record_parse_error(&mut self) {
        self.packets_parse_error_total += 1;
    }

    pub fn record_signed_packet_observed(&mut self) {
        self.packets_signed_observed_total += 1;
    }

    pub fn record_signed_packet_valid(&mut self) {
        self.packets_signed_valid_total += 1;
    }

    pub fn record_signed_packet_invalid(&mut self) {
        self.packets_signed_invalid_total += 1;
    }

    pub fn record_unsigned_packet_rejected(&mut self) {
        self.packets_unsigned_rejected_total += 1;
    }

    pub fn record_signing_replay_rejected(&mut self) {
        self.signing_replay_rejected_total += 1;
    }

    pub fn record_setup_signing_observed(&mut self) {
        self.setup_signing_observed_total += 1;
    }

    pub fn record_critical_command_observed(&mut self) {
        self.commands_critical_observed_total += 1;
    }

    pub fn record_processing_latency_us(&mut self, latency_us: u64) {
        self.processing_latency_samples += 1;
        self.processing_latency_total_us += latency_us;
        self.processing_latency_max_us = self.processing_latency_max_us.max(latency_us);
    }

    pub fn record_parse_latency_us(&mut self, latency_us: u64) {
        self.parse_latency_samples += 1;
        self.parse_latency_total_us += latency_us;
        self.parse_latency_max_us = self.parse_latency_max_us.max(latency_us);
    }

    pub fn record_policy_latency_us(&mut self, latency_us: u64) {
        self.policy_latency_samples += 1;
        self.policy_latency_total_us += latency_us;
        self.policy_latency_max_us = self.policy_latency_max_us.max(latency_us);
    }

    pub fn set_last_heartbeat_age_ms(&mut self, age_ms: u64) {
        self.last_heartbeat_age_ms = Some(age_ms);
    }
}
