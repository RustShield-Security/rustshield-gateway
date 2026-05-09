use std::{
    future::Future,
    net::{IpAddr, SocketAddr},
    sync::Arc,
    time::Instant,
};

use thiserror::Error;
use tokio::{net::UdpSocket, sync::Mutex};

use crate::{
    config::{AppConfig, SigningPolicy, UnknownModePolicy},
    flight_state::{FlightModeClassification, FlightState},
    mavlink_codec::{decode_datagram, parse_error_decision, MavlinkSemantic},
    metrics::GatewayCounters,
    observability::{spawn_readonly_endpoint, ReadonlyEndpoint},
    security_filter::{
        evaluate_command, Direction, MessageContext, PolicyDecision, SecurityPolicy, TransportKind,
    },
    signing::{SigningError, SigningRejectReason, SigningValidation, SigningValidator},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportStatus {
    NotStarted,
    Running,
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("could not bind UDP socket {addr}: {source}")]
    BindUdp {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("UDP transport error: {0}")]
    Io(#[from] std::io::Error),

    #[error("signing configuration error: {0}")]
    Signing(#[from] SigningError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardTarget {
    Gcs,
    Vehicle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatagramOutcome {
    Forward { target: ForwardTarget },
    DropInvalid,
    Blocked,
}

#[derive(Debug, Clone, Copy)]
enum UdpFlow {
    GcsToVehicle,
    VehicleToGcs,
}

impl UdpFlow {
    fn as_str(self) -> &'static str {
        match self {
            Self::GcsToVehicle => "gcs_to_vehicle",
            Self::VehicleToGcs => "vehicle_to_gcs",
        }
    }
}

struct UdpFlowRuntime {
    flow: UdpFlow,
    receive_socket: Arc<UdpSocket>,
    send_socket: Arc<UdpSocket>,
    destination: SocketAddr,
    max_datagram_size: usize,
    flight_state: Arc<Mutex<FlightState>>,
    counters: Arc<Mutex<GatewayCounters>>,
    policy: SecurityPolicy,
    signing_policy: SigningPolicy,
    signing_validator: Option<Arc<SigningValidator>>,
}

#[derive(Clone, Copy)]
struct SigningAuditContext<'a> {
    policy: SigningPolicy,
    validator: Option<&'a SigningValidator>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SigningAssessment {
    NotRequired,
    Unsigned,
    Valid,
    Invalid { reason: SigningRejectReason },
}

impl SigningAssessment {
    fn is_valid(self) -> bool {
        matches!(self, Self::Valid)
    }
}

pub struct UdpGateway {
    gcs_socket: Arc<UdpSocket>,
    vehicle_socket: Arc<UdpSocket>,
    config: AppConfig,
    policy: SecurityPolicy,
    signing_policy: SigningPolicy,
    signing_validator: Option<Arc<SigningValidator>>,
    flight_state: Arc<Mutex<FlightState>>,
    counters: Arc<Mutex<GatewayCounters>>,
    readonly_endpoint: Option<ReadonlyEndpoint>,
}

impl UdpGateway {
    pub async fn bind(config: AppConfig) -> Result<Self, TransportError> {
        let gcs_socket = UdpSocket::bind(config.udp.listen_gcs)
            .await
            .map_err(|source| TransportError::BindUdp {
                addr: config.udp.listen_gcs,
                source,
            })?;
        let vehicle_socket =
            UdpSocket::bind(config.udp.listen_vehicle)
                .await
                .map_err(|source| TransportError::BindUdp {
                    addr: config.udp.listen_vehicle,
                    source,
                })?;

        tracing::info!(
            event = "transport.opened",
            transport = "udp",
            listen_gcs = %gcs_socket.local_addr()?,
            listen_vehicle = %vehicle_socket.local_addr()?,
            vehicle_addr = %config.udp.vehicle_addr,
            gcs_addr = %config.udp.gcs_addr,
            "UDP transport opened"
        );

        let policy = security_policy_from_config(&config);
        let signing_policy = config.signing.policy;
        let signing_validator = signing_validator_from_config(&config)?;
        let counters = Arc::new(Mutex::new(GatewayCounters::default()));
        let readonly_endpoint = if config.metrics.enabled {
            match config.metrics.readonly_bind {
                Some(bind_addr) => {
                    let endpoint =
                        spawn_readonly_endpoint(bind_addr, Arc::clone(&counters), Instant::now())
                            .await?;
                    tracing::info!(
                        event = "observability.readonly_opened",
                        bind_addr = %endpoint.local_addr,
                        paths = "/healthz,/metrics",
                        read_only = true,
                        "read-only observability endpoint opened"
                    );
                    Some(endpoint)
                }
                None => None,
            }
        } else {
            None
        };

        Ok(Self {
            gcs_socket: Arc::new(gcs_socket),
            vehicle_socket: Arc::new(vehicle_socket),
            config,
            policy,
            signing_policy,
            signing_validator,
            flight_state: Arc::new(Mutex::new(FlightState::default())),
            counters,
            readonly_endpoint,
        })
    }

    pub fn gcs_listen_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.gcs_socket.local_addr()
    }

    pub fn vehicle_listen_addr(&self) -> Result<SocketAddr, std::io::Error> {
        self.vehicle_socket.local_addr()
    }

    pub async fn counters(&self) -> GatewayCounters {
        *self.counters.lock().await
    }

    pub async fn run_until_shutdown(
        self,
        shutdown: impl Future<Output = std::io::Result<()>>,
    ) -> Result<(), TransportError> {
        let mut gcs_task = tokio::spawn(run_udp_flow(UdpFlowRuntime {
            flow: UdpFlow::GcsToVehicle,
            receive_socket: Arc::clone(&self.gcs_socket),
            send_socket: Arc::clone(&self.vehicle_socket),
            destination: self.config.udp.vehicle_addr,
            max_datagram_size: self.config.udp.max_datagram_size,
            flight_state: Arc::clone(&self.flight_state),
            counters: Arc::clone(&self.counters),
            policy: self.policy.clone(),
            signing_policy: self.signing_policy,
            signing_validator: self.signing_validator.as_ref().map(Arc::clone),
        }));
        let mut vehicle_task = tokio::spawn(run_udp_flow(UdpFlowRuntime {
            flow: UdpFlow::VehicleToGcs,
            receive_socket: Arc::clone(&self.vehicle_socket),
            send_socket: Arc::clone(&self.gcs_socket),
            destination: self.config.udp.gcs_addr,
            max_datagram_size: self.config.udp.max_datagram_size,
            flight_state: Arc::clone(&self.flight_state),
            counters: Arc::clone(&self.counters),
            policy: self.policy.clone(),
            signing_policy: self.signing_policy,
            signing_validator: self.signing_validator.as_ref().map(Arc::clone),
        }));

        let result = tokio::select! {
            result = &mut gcs_task => {
                vehicle_task.abort();
                result.expect("UDP GCS task must not panic")
            }
            result = &mut vehicle_task => {
                gcs_task.abort();
                result.expect("UDP vehicle task must not panic")
            }
            result = shutdown => {
                gcs_task.abort();
                vehicle_task.abort();
                result.map_err(TransportError::Io)
            }
        };

        if let Some(endpoint) = &self.readonly_endpoint {
            endpoint.task.abort();
        }
        log_metrics_snapshot(&self.counters).await;

        result
    }
}

async fn log_metrics_snapshot(counters: &Mutex<GatewayCounters>) {
    let counters = *counters.lock().await;
    tracing::info!(
        event = "metrics.snapshot",
        packets_received_total = counters.packets_received_total,
        packets_forwarded_total = counters.packets_forwarded_total,
        packets_blocked_total = counters.packets_blocked_total,
        packets_parse_error_total = counters.packets_parse_error_total,
        packets_signed_observed_total = counters.packets_signed_observed_total,
        packets_signed_valid_total = counters.packets_signed_valid_total,
        packets_signed_invalid_total = counters.packets_signed_invalid_total,
        packets_unsigned_rejected_total = counters.packets_unsigned_rejected_total,
        signing_replay_rejected_total = counters.signing_replay_rejected_total,
        setup_signing_observed_total = counters.setup_signing_observed_total,
        shadow_policy_would_block_total = counters.shadow_policy_would_block_total,
        shadow_signing_would_reject_total = counters.shadow_signing_would_reject_total,
        shadow_unsigned_critical_total = counters.shadow_unsigned_critical_total,
        shadow_invalid_signature_total = counters.shadow_invalid_signature_total,
        shadow_replay_total = counters.shadow_replay_total,
        commands_critical_observed_total = counters.commands_critical_observed_total,
        processing_latency_samples = counters.processing_latency_samples,
        processing_latency_total_us = counters.processing_latency_total_us,
        processing_latency_max_us = counters.processing_latency_max_us,
        parse_latency_samples = counters.parse_latency_samples,
        parse_latency_total_us = counters.parse_latency_total_us,
        parse_latency_max_us = counters.parse_latency_max_us,
        policy_latency_samples = counters.policy_latency_samples,
        policy_latency_total_us = counters.policy_latency_total_us,
        policy_latency_max_us = counters.policy_latency_max_us,
        "gateway metrics snapshot"
    );
}

async fn run_udp_flow(runtime: UdpFlowRuntime) -> Result<(), TransportError> {
    let mut buffer = vec![0_u8; runtime.max_datagram_size];

    loop {
        let (len, source) = runtime.receive_socket.recv_from(&mut buffer).await?;
        let datagram = &buffer[..len];
        let outcome = process_datagram(
            runtime.flow,
            source.ip(),
            datagram,
            &runtime.flight_state,
            &runtime.counters,
            &runtime.policy,
            SigningAuditContext {
                policy: runtime.signing_policy,
                validator: runtime.signing_validator.as_deref(),
            },
        )
        .await;

        if matches!(outcome, DatagramOutcome::Forward { .. }) {
            runtime
                .send_socket
                .send_to(datagram, runtime.destination)
                .await?;
            runtime.counters.lock().await.record_forwarded_packet();
        }
    }
}

async fn process_datagram(
    flow: UdpFlow,
    source_ip: IpAddr,
    datagram: &[u8],
    flight_state: &Mutex<FlightState>,
    counters: &Mutex<GatewayCounters>,
    policy: &SecurityPolicy,
    signing: SigningAuditContext<'_>,
) -> DatagramOutcome {
    let processing_started = Instant::now();
    counters.lock().await.record_received_packet();

    let parse_started = Instant::now();
    let packet = match decode_datagram(datagram) {
        Ok(packet) => {
            counters
                .lock()
                .await
                .record_parse_latency_us(elapsed_us(parse_started));
            packet
        }
        Err(error) => {
            {
                let mut counters = counters.lock().await;
                counters.record_parse_latency_us(elapsed_us(parse_started));
                counters.record_parse_error();
                counters.record_processing_latency_us(elapsed_us(processing_started));
            }
            let decision = parse_error_decision();
            if let PolicyDecision::DropInvalid(reason) = decision {
                tracing::warn!(
                    event = reason.audit_event,
                    rule_id = reason.rule_id.as_str(),
                    reason = reason.reason,
                    error = %error,
                    latency_us = elapsed_us(processing_started),
                    "dropping invalid MAVLink datagram"
                );
            }
            return DatagramOutcome::DropInvalid;
        }
    };

    if matches!(packet.semantic, MavlinkSemantic::SetupSigning) {
        {
            let mut counters = counters.lock().await;
            counters.record_blocked_packet();
            counters.record_setup_signing_observed();
            counters.record_processing_latency_us(elapsed_us(processing_started));
        }
        tracing::warn!(
            event = "mavlink.setup_signing_observed",
            direction = flow.as_str(),
            message_id = packet.frame.message_id,
            system_id = packet.frame.system_id,
            component_id = packet.frame.component_id,
            source_ip = %source_ip,
            "SETUP_SIGNING blocked; signing key provisioning is not forwarded automatically"
        );
        return DatagramOutcome::Blocked;
    }

    match flow {
        UdpFlow::VehicleToGcs => {
            inspect_signing(flow, source_ip, datagram, &packet, counters, signing).await;

            if let MavlinkSemantic::Heartbeat(observation) = packet.semantic {
                let mut state = flight_state.lock().await;
                if let Some(event) = state.update_heartbeat(observation) {
                    tracing::info!(
                        event = event.audit_event,
                        autopilot = ?event.autopilot,
                        vehicle_family = ?event.vehicle_family,
                        custom_mode = ?event.custom_mode,
                        mode_name = ?event.mode_name,
                        classification = ?event.classification,
                        previous_classification = ?event.previous_classification,
                        source_system = event.source_system,
                        source_component = event.source_component,
                        "flight mode changed"
                    );
                }
            }

            counters
                .lock()
                .await
                .record_processing_latency_us(elapsed_us(processing_started));
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs,
            }
        }
        UdpFlow::GcsToVehicle => {
            let MavlinkSemantic::Command(ref command) = packet.semantic else {
                inspect_signing(flow, source_ip, datagram, &packet, counters, signing).await;
                counters
                    .lock()
                    .await
                    .record_processing_latency_us(elapsed_us(processing_started));
                return DatagramOutcome::Forward {
                    target: ForwardTarget::Vehicle,
                };
            };

            let is_critical_or_high_risk =
                command.is_cataloged_critical_command() || command.is_cataloged_high_risk_command();
            if is_critical_or_high_risk {
                tracing::info!(
                    event = "security.command_observed",
                    message_id = command.message_id,
                    command = command.command,
                    source_ip = %source_ip,
                    "critical or high-risk command observed"
                );
            }

            let requires_signing = is_critical_or_high_risk;
            let signing_assessment = if requires_signing && signing.policy == SigningPolicy::Enforce
            {
                enforce_signing_for_command(
                    flow, source_ip, datagram, &packet, command, counters, signing,
                )
                .await
            } else {
                inspect_signing(flow, source_ip, datagram, &packet, counters, signing).await
            };
            if policy.shadow_enforce && signing.policy != SigningPolicy::Enforce && requires_signing
            {
                record_shadow_signing_for_command(
                    flow,
                    source_ip,
                    &packet,
                    command,
                    counters,
                    signing.policy,
                    signing_assessment,
                )
                .await;
            }
            if requires_signing
                && signing.policy == SigningPolicy::Enforce
                && !signing_assessment.is_valid()
            {
                {
                    let mut counters = counters.lock().await;
                    if is_critical_or_high_risk {
                        counters.record_critical_command_observed();
                    }
                    counters.record_blocked_packet();
                    counters.record_processing_latency_us(elapsed_us(processing_started));
                }
                return DatagramOutcome::Blocked;
            }

            let flight_mode = flight_state
                .lock()
                .await
                .current_mode()
                .map_or(FlightModeClassification::Unknown, |snapshot| {
                    snapshot.classification
                });
            let context = MessageContext {
                direction: Direction::GcsToVehicle,
                transport: TransportKind::Udp,
                source_ip: Some(source_ip),
                flight_mode,
            };

            if policy.shadow_enforce {
                record_shadow_policy_for_command(
                    flow, source_ip, &packet, command, counters, policy, &context,
                )
                .await;
            }
            let policy_started = Instant::now();
            let decision = evaluate_command(policy, &context, command);
            let policy_latency_us = elapsed_us(policy_started);

            match decision {
                PolicyDecision::Allow => {
                    let mut counters = counters.lock().await;
                    if is_critical_or_high_risk {
                        counters.record_critical_command_observed();
                    }
                    counters.record_policy_latency_us(policy_latency_us);
                    counters.record_processing_latency_us(elapsed_us(processing_started));
                    DatagramOutcome::Forward {
                        target: ForwardTarget::Vehicle,
                    }
                }
                PolicyDecision::AuditOnly(reason) => {
                    tracing::warn!(
                        event = "security.audit_only",
                        rule_id = reason.rule_id.as_str(),
                        severity = ?reason.severity,
                        param2_class = ?reason.param2_class,
                        message_id = command.message_id,
                        command = command.command,
                        source_ip = %source_ip,
                        flight_mode = ?flight_mode,
                        latency_us = elapsed_us(processing_started),
                        "command would be blocked by policy"
                    );
                    let mut counters = counters.lock().await;
                    if is_critical_or_high_risk {
                        counters.record_critical_command_observed();
                    }
                    counters.record_policy_latency_us(policy_latency_us);
                    counters.record_processing_latency_us(elapsed_us(processing_started));
                    DatagramOutcome::Forward {
                        target: ForwardTarget::Vehicle,
                    }
                }
                PolicyDecision::Block(reason) => {
                    {
                        let mut counters = counters.lock().await;
                        if is_critical_or_high_risk {
                            counters.record_critical_command_observed();
                        }
                        counters.record_policy_latency_us(policy_latency_us);
                        counters.record_blocked_packet();
                        counters.record_processing_latency_us(elapsed_us(processing_started));
                    }
                    tracing::warn!(
                        event = reason.audit_event,
                        rule_id = reason.rule_id.as_str(),
                        severity = ?reason.severity,
                        param2_class = ?reason.param2_class,
                        message_id = command.message_id,
                        command = command.command,
                        source_ip = %source_ip,
                        flight_mode = ?flight_mode,
                        latency_us = elapsed_us(processing_started),
                        "command blocked by policy"
                    );
                    DatagramOutcome::Blocked
                }
                PolicyDecision::DropInvalid(_) => {
                    let mut counters = counters.lock().await;
                    if is_critical_or_high_risk {
                        counters.record_critical_command_observed();
                    }
                    counters.record_policy_latency_us(policy_latency_us);
                    counters.record_processing_latency_us(elapsed_us(processing_started));
                    DatagramOutcome::DropInvalid
                }
            }
        }
    }
}

fn elapsed_us(started: Instant) -> u64 {
    started.elapsed().as_micros().try_into().unwrap_or(u64::MAX)
}

async fn inspect_signing(
    flow: UdpFlow,
    source_ip: IpAddr,
    datagram: &[u8],
    packet: &crate::mavlink_codec::ParsedMavlinkPacket,
    counters: &Mutex<GatewayCounters>,
    signing: SigningAuditContext<'_>,
) -> SigningAssessment {
    if !packet.frame.signed {
        if signing.policy == SigningPolicy::Audit
            && matches!(
                &packet.semantic,
                MavlinkSemantic::Command(command)
                    if command.is_cataloged_critical_command()
                        || command.is_cataloged_high_risk_command()
            )
        {
            tracing::warn!(
                event = "mavlink.signing_rejected",
                direction = flow.as_str(),
                message_id = packet.frame.message_id,
                system_id = packet.frame.system_id,
                component_id = packet.frame.component_id,
                source_ip = %source_ip,
                authenticated = false,
                signing_policy = "audit",
                reason = "unsigned",
                "unsigned critical or high-risk command observed in signing audit mode"
            );
        }
        return SigningAssessment::Unsigned;
    }

    counters.lock().await.record_signed_packet_observed();

    match signing.policy {
        SigningPolicy::Observe => {
            tracing::info!(
                event = "mavlink.signed_observed",
                direction = flow.as_str(),
                message_id = packet.frame.message_id,
                system_id = packet.frame.system_id,
                component_id = packet.frame.component_id,
                source_ip = %source_ip,
                authenticated = false,
                signing_policy = "observe",
                "MAVLink signed packet observed without cryptographic validation"
            );
            SigningAssessment::NotRequired
        }
        SigningPolicy::Audit | SigningPolicy::Enforce => {
            let Some(validator) = signing.validator else {
                tracing::warn!(
                    event = "mavlink.signing_rejected",
                    direction = flow.as_str(),
                    message_id = packet.frame.message_id,
                    system_id = packet.frame.system_id,
                    component_id = packet.frame.component_id,
                    source_ip = %source_ip,
                    authenticated = false,
                    signing_policy = signing.policy.as_str(),
                    reason = "validator_unavailable",
                    "signed MAVLink packet could not be validated"
                );
                return SigningAssessment::Invalid {
                    reason: SigningRejectReason::ParseFailed,
                };
            };

            match validator.validate_datagram(datagram) {
                SigningValidation::Valid(info) => {
                    counters.lock().await.record_signed_packet_valid();
                    tracing::info!(
                        event = "mavlink.signing_validated",
                        direction = flow.as_str(),
                        message_id = packet.frame.message_id,
                        system_id = packet.frame.system_id,
                        component_id = packet.frame.component_id,
                        source_ip = %source_ip,
                        authenticated = true,
                        signing_policy = signing.policy.as_str(),
                        link_id = info.link_id,
                        timestamp = info.timestamp,
                        "MAVLink signature validated"
                    );
                    SigningAssessment::Valid
                }
                SigningValidation::Invalid { info, reason } => {
                    let mut counters = counters.lock().await;
                    counters.record_signed_packet_invalid();
                    if reason == SigningRejectReason::Replay {
                        counters.record_signing_replay_rejected();
                    }
                    drop(counters);

                    tracing::warn!(
                        event = "mavlink.signing_rejected",
                        direction = flow.as_str(),
                        message_id = packet.frame.message_id,
                        system_id = packet.frame.system_id,
                        component_id = packet.frame.component_id,
                        source_ip = %source_ip,
                        authenticated = false,
                        signing_policy = signing.policy.as_str(),
                        link_id = info.map(|info| info.link_id),
                        timestamp = info.map(|info| info.timestamp),
                        reason = reason.as_str(),
                        "MAVLink signature rejected"
                    );
                    SigningAssessment::Invalid { reason }
                }
                SigningValidation::Unsigned => {
                    tracing::warn!(
                        event = "mavlink.signing_rejected",
                        direction = flow.as_str(),
                        message_id = packet.frame.message_id,
                        system_id = packet.frame.system_id,
                        component_id = packet.frame.component_id,
                        source_ip = %source_ip,
                        authenticated = false,
                        signing_policy = signing.policy.as_str(),
                        reason = "unsigned",
                        "unsigned MAVLink packet observed in signing validation mode"
                    );
                    SigningAssessment::Unsigned
                }
            }
        }
    }
}

async fn enforce_signing_for_command(
    flow: UdpFlow,
    source_ip: IpAddr,
    datagram: &[u8],
    packet: &crate::mavlink_codec::ParsedMavlinkPacket,
    command: &crate::security_filter::CommandMessage,
    counters: &Mutex<GatewayCounters>,
    signing: SigningAuditContext<'_>,
) -> SigningAssessment {
    let assessment = inspect_signing(flow, source_ip, datagram, packet, counters, signing).await;
    if assessment.is_valid() {
        return assessment;
    }

    if matches!(assessment, SigningAssessment::Unsigned) {
        counters.lock().await.record_unsigned_packet_rejected();
    }

    let reason = match assessment {
        SigningAssessment::NotRequired => "validation_not_required",
        SigningAssessment::Unsigned => "unsigned",
        SigningAssessment::Valid => "valid",
        SigningAssessment::Invalid { reason } => reason.as_str(),
    };

    tracing::warn!(
        event = "mavlink.signing_rejected",
        direction = flow.as_str(),
        message_id = packet.frame.message_id,
        command = command.command,
        system_id = packet.frame.system_id,
        component_id = packet.frame.component_id,
        source_ip = %source_ip,
        authenticated = false,
        signing_policy = "enforce",
        reason,
        "critical or high-risk command blocked by MAVLink signing enforce policy"
    );

    assessment
}

async fn record_shadow_signing_for_command(
    flow: UdpFlow,
    source_ip: IpAddr,
    packet: &crate::mavlink_codec::ParsedMavlinkPacket,
    command: &crate::security_filter::CommandMessage,
    counters: &Mutex<GatewayCounters>,
    signing_policy: SigningPolicy,
    assessment: SigningAssessment,
) {
    let reason = match assessment {
        SigningAssessment::Unsigned => "unsigned",
        SigningAssessment::Invalid { reason } => reason.as_str(),
        SigningAssessment::NotRequired | SigningAssessment::Valid => return,
    };

    {
        let mut counters = counters.lock().await;
        counters.record_shadow_signing_would_reject();
        match assessment {
            SigningAssessment::Unsigned => counters.record_shadow_unsigned_critical(),
            SigningAssessment::Invalid {
                reason: SigningRejectReason::Replay,
            } => counters.record_shadow_replay(),
            SigningAssessment::Invalid { .. } => counters.record_shadow_invalid_signature(),
            SigningAssessment::NotRequired | SigningAssessment::Valid => {}
        }
    }

    tracing::warn!(
        event = "signing.shadow_would_reject",
        direction = flow.as_str(),
        message_id = packet.frame.message_id,
        command = command.command,
        system_id = packet.frame.system_id,
        component_id = packet.frame.component_id,
        source_ip = %source_ip,
        authenticated = false,
        signing_policy = signing_policy.as_str(),
        shadow_enforce = true,
        reason,
        "critical or high-risk command would be rejected by signing enforce"
    );
}

async fn record_shadow_policy_for_command(
    flow: UdpFlow,
    source_ip: IpAddr,
    packet: &crate::mavlink_codec::ParsedMavlinkPacket,
    command: &crate::security_filter::CommandMessage,
    counters: &Mutex<GatewayCounters>,
    policy: &SecurityPolicy,
    context: &MessageContext,
) {
    let mut shadow_policy = policy.clone();
    shadow_policy.audit_only = false;
    shadow_policy.shadow_enforce = false;

    let PolicyDecision::Block(reason) = evaluate_command(&shadow_policy, context, command) else {
        return;
    };

    counters.lock().await.record_shadow_policy_would_block();
    tracing::warn!(
        event = "security.shadow_would_block",
        direction = flow.as_str(),
        rule_id = reason.rule_id.as_str(),
        severity = ?reason.severity,
        param2_class = ?reason.param2_class,
        message_id = packet.frame.message_id,
        command = command.command,
        system_id = packet.frame.system_id,
        component_id = packet.frame.component_id,
        source_ip = %source_ip,
        flight_mode = ?context.flight_mode,
        shadow_enforce = true,
        "command would be blocked by semantic policy under shadow enforce"
    );
}

fn security_policy_from_config(config: &AppConfig) -> SecurityPolicy {
    SecurityPolicy {
        certified_ips: config.security.certified_ips.clone(),
        audit_only: config.security.audit_only
            || config.security.unknown_mode_policy == UnknownModePolicy::AuditOnly,
        shadow_enforce: config.security.shadow_enforce,
        block_arm_in_auto_mode: config.security.block_arm_in_auto_mode,
        block_critical_when_mode_unknown: config.security.unknown_mode_policy
            != UnknownModePolicy::Allow,
    }
}

fn signing_validator_from_config(
    config: &AppConfig,
) -> Result<Option<Arc<SigningValidator>>, SigningError> {
    match config.signing.policy {
        SigningPolicy::Observe => Ok(None),
        SigningPolicy::Audit | SigningPolicy::Enforce => {
            let path = config
                .signing
                .key_path
                .as_ref()
                .expect("config validation requires signing.key_path for audit/enforce");
            Ok(Some(Arc::new(SigningValidator::from_key_file(
                path,
                config.signing.link_id,
            )?)))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::Write,
        sync::{Arc as StdArc, Mutex as StdMutex},
        time::Duration,
    };

    use mavlink::{common, MavHeader};
    use tokio::time::timeout;
    use tracing_subscriber::fmt::MakeWriter;

    use super::*;

    fn fixture(message: &common::MavMessage) -> Vec<u8> {
        fixture_with_header(
            message,
            MavHeader {
                sequence: 7,
                system_id: 1,
                component_id: 1,
            },
        )
    }

    fn fixture_with_header(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v2_msg(&mut bytes, header, message).expect("fixture serializes");
        bytes
    }

    fn v1_fixture_with_header(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v1_msg(&mut bytes, header, message).expect("MAVLink v1 fixture serializes");
        bytes
    }

    fn heartbeat(custom_mode: u32) -> Vec<u8> {
        fixture(&common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        }))
    }

    fn px4_heartbeat(custom_mode: u32) -> Vec<u8> {
        fixture(&common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_PX4,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        }))
    }

    fn arm_command_message() -> common::MavMessage {
        common::MavMessage::COMMAND_LONG(common::COMMAND_LONG_DATA {
            param1: 1.0,
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: common::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
            target_system: 1,
            target_component: 1,
            confirmation: 0,
        })
    }

    fn arm_command() -> Vec<u8> {
        fixture(&arm_command_message())
    }

    fn arm_command_v1() -> Vec<u8> {
        v1_fixture_with_header(
            &arm_command_message(),
            MavHeader {
                sequence: 7,
                system_id: 1,
                component_id: 1,
            },
        )
    }

    fn takeoff_command_message() -> common::MavMessage {
        common::MavMessage::COMMAND_LONG(common::COMMAND_LONG_DATA {
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: common::MavCmd::MAV_CMD_NAV_TAKEOFF,
            target_system: 1,
            target_component: 1,
            confirmation: 0,
        })
    }

    fn takeoff_command() -> Vec<u8> {
        fixture(&takeoff_command_message())
    }

    fn signed_arm_command() -> Vec<u8> {
        signed_fixture(&arm_command_message())
    }

    fn request_message_command() -> Vec<u8> {
        fixture(&request_message())
    }

    fn setup_signing() -> Vec<u8> {
        fixture(&common::MavMessage::SETUP_SIGNING(
            common::SETUP_SIGNING_DATA::default(),
        ))
    }

    fn request_message() -> common::MavMessage {
        common::MavMessage::COMMAND_LONG(common::COMMAND_LONG_DATA {
            param1: 0.0,
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: common::MavCmd::MAV_CMD_REQUEST_MESSAGE,
            target_system: 77,
            target_component: 88,
            confirmation: 0,
        })
    }

    fn signed_fixture(message: &common::MavMessage) -> Vec<u8> {
        signed_fixture_with_header(
            message,
            MavHeader {
                sequence: 8,
                system_id: 1,
                component_id: 1,
            },
        )
    }

    fn signed_fixture_with_header(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
        let mut raw = mavlink::MAVLinkV2MessageRaw::new();
        raw.serialize_message_for_signing(header, message);
        *raw.signature_link_id_mut() = 7;
        raw.signature_timestamp_bytes_mut()
            .copy_from_slice(&[0xff; 6]);
        let mut signature = [0_u8; 6];
        raw.calculate_signature(&crate::signing::INSECURE_TEST_SIGNING_KEY, &mut signature);
        raw.signature_value_mut().copy_from_slice(&signature);
        raw.raw_bytes().to_vec()
    }

    fn signed_heartbeat(custom_mode: u32) -> Vec<u8> {
        let message = common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        });
        signed_fixture(&message)
    }

    fn signed_px4_heartbeat(custom_mode: u32) -> Vec<u8> {
        let message = common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_PX4,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        });
        signed_fixture(&message)
    }

    fn tampered_signature(mut packet: Vec<u8>) -> Vec<u8> {
        let last = packet.last_mut().expect("signed fixture has signature");
        *last ^= 0x01;
        packet
    }

    fn no_signing_validator() -> Option<Arc<SigningValidator>> {
        None
    }

    fn test_config(gcs_addr: SocketAddr, vehicle_addr: SocketAddr) -> AppConfig {
        let mut config = AppConfig::default();
        config.udp.listen_gcs = "127.0.0.1:0".parse().expect("valid socket");
        config.udp.listen_vehicle = "127.0.0.1:0".parse().expect("valid socket");
        config.udp.gcs_addr = gcs_addr;
        config.udp.vehicle_addr = vehicle_addr;
        config.security.certified_ips = vec!["127.0.0.2".parse().expect("valid IP")];
        config
    }

    fn test_policy() -> SecurityPolicy {
        SecurityPolicy {
            certified_ips: vec!["127.0.0.2".parse().expect("valid IP")],
            audit_only: false,
            shadow_enforce: false,
            block_arm_in_auto_mode: true,
            block_critical_when_mode_unknown: true,
        }
    }

    fn certified_loopback_policy() -> SecurityPolicy {
        SecurityPolicy {
            certified_ips: vec!["127.0.0.1".parse().expect("valid IP")],
            audit_only: false,
            shadow_enforce: false,
            block_arm_in_auto_mode: true,
            block_critical_when_mode_unknown: true,
        }
    }

    #[tokio::test]
    async fn forwards_vehicle_heartbeat_to_gcs() {
        let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind GCS");
        let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind vehicle");
        let gateway = UdpGateway::bind(test_config(
            gcs_receiver.local_addr().expect("GCS addr"),
            vehicle_receiver.local_addr().expect("vehicle addr"),
        ))
        .await
        .expect("gateway binds");
        let vehicle_gateway_addr = gateway.vehicle_listen_addr().expect("vehicle listen addr");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(gateway.run_until_shutdown(async {
            let _ = shutdown_rx.await;
            Ok(())
        }));

        let sitl = UdpSocket::bind("127.0.0.1:0").await.expect("bind SITL");
        let packet = heartbeat(3);
        sitl.send_to(&packet, vehicle_gateway_addr)
            .await
            .expect("send heartbeat");

        let mut buffer = vec![0_u8; 2048];
        let (len, _) = timeout(Duration::from_secs(1), gcs_receiver.recv_from(&mut buffer))
            .await
            .expect("GCS receives packet")
            .expect("recv succeeds");
        assert_eq!(&buffer[..len], packet.as_slice());

        let _ = shutdown_tx.send(());
        task.await.expect("task joins").expect("gateway stops");
    }

    #[tokio::test]
    async fn forwards_px4_vehicle_heartbeat_to_gcs_without_modifying_bytes() {
        let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind GCS");
        let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind vehicle");
        let gateway = UdpGateway::bind(test_config(
            gcs_receiver.local_addr().expect("GCS addr"),
            vehicle_receiver.local_addr().expect("vehicle addr"),
        ))
        .await
        .expect("gateway binds");
        let vehicle_gateway_addr = gateway.vehicle_listen_addr().expect("vehicle listen addr");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(gateway.run_until_shutdown(async {
            let _ = shutdown_rx.await;
            Ok(())
        }));

        let sitl = UdpSocket::bind("127.0.0.1:0").await.expect("bind PX4 SITL");
        let packet = px4_heartbeat(4);
        sitl.send_to(&packet, vehicle_gateway_addr)
            .await
            .expect("send PX4 heartbeat");

        let mut buffer = vec![0_u8; 2048];
        let (len, _) = timeout(Duration::from_secs(1), gcs_receiver.recv_from(&mut buffer))
            .await
            .expect("GCS receives PX4 packet")
            .expect("recv succeeds");
        assert_eq!(&buffer[..len], packet.as_slice());

        let _ = shutdown_tx.send(());
        task.await.expect("task joins").expect("gateway stops");
    }

    #[tokio::test]
    async fn px4_heartbeat_keeps_mode_unknown_and_blocks_critical_command() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let heartbeat_outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &px4_heartbeat(4),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        assert_eq!(
            heartbeat_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        assert_eq!(
            flight_state
                .lock()
                .await
                .current_mode()
                .expect("PX4 heartbeat sets state")
                .classification,
            FlightModeClassification::Unknown
        );

        let command_outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;

        assert_eq!(command_outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.commands_critical_observed_total, 1);
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_forwarded_total, 0);
    }

    #[tokio::test]
    async fn px4_signed_heartbeat_is_observed_with_existing_signing_rules() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &signed_px4_heartbeat(4),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;

        assert_eq!(
            outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_valid_total, 0);
        assert_eq!(counters.packets_signed_invalid_total, 0);
        assert_eq!(counters.packets_blocked_total, 0);
    }

    #[tokio::test]
    async fn blocks_gcs_arm_command_when_mode_is_unknown() {
        let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind GCS");
        let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind vehicle");
        let gateway = UdpGateway::bind(test_config(
            gcs_receiver.local_addr().expect("GCS addr"),
            vehicle_receiver.local_addr().expect("vehicle addr"),
        ))
        .await
        .expect("gateway binds");
        let gcs_gateway_addr = gateway.gcs_listen_addr().expect("GCS listen addr");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(gateway.run_until_shutdown(async {
            let _ = shutdown_rx.await;
            Ok(())
        }));

        let gcs = UdpSocket::bind("127.0.0.1:0").await.expect("bind sender");
        gcs.send_to(&arm_command(), gcs_gateway_addr)
            .await
            .expect("send command");

        let mut buffer = vec![0_u8; 2048];
        let received = timeout(
            Duration::from_millis(200),
            vehicle_receiver.recv_from(&mut buffer),
        )
        .await;
        assert!(
            received.is_err(),
            "vehicle must not receive blocked command"
        );

        let _ = shutdown_tx.send(());
        task.await.expect("task joins").expect("gateway stops");
    }

    #[tokio::test]
    async fn forwards_allowed_gcs_command_to_vehicle() {
        let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind GCS");
        let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind vehicle");
        let gateway = UdpGateway::bind(test_config(
            gcs_receiver.local_addr().expect("GCS addr"),
            vehicle_receiver.local_addr().expect("vehicle addr"),
        ))
        .await
        .expect("gateway binds");
        let gcs_gateway_addr = gateway.gcs_listen_addr().expect("GCS listen addr");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(gateway.run_until_shutdown(async {
            let _ = shutdown_rx.await;
            Ok(())
        }));

        let gcs = UdpSocket::bind("127.0.0.1:0").await.expect("bind sender");
        let packet = request_message_command();
        gcs.send_to(&packet, gcs_gateway_addr)
            .await
            .expect("send command");

        let mut buffer = vec![0_u8; 2048];
        let (len, _) = timeout(
            Duration::from_secs(1),
            vehicle_receiver.recv_from(&mut buffer),
        )
        .await
        .expect("vehicle receives allowed command")
        .expect("recv succeeds");
        assert_eq!(&buffer[..len], packet.as_slice());

        let _ = shutdown_tx.send(());
        task.await.expect("task joins").expect("gateway stops");
    }

    #[tokio::test]
    async fn forwards_mavlink_v1_allowed_command_without_modifying_bytes() {
        let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind GCS");
        let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind vehicle");
        let gateway = UdpGateway::bind(test_config(
            gcs_receiver.local_addr().expect("GCS addr"),
            vehicle_receiver.local_addr().expect("vehicle addr"),
        ))
        .await
        .expect("gateway binds");
        let gcs_gateway_addr = gateway.gcs_listen_addr().expect("GCS listen addr");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(gateway.run_until_shutdown(async {
            let _ = shutdown_rx.await;
            Ok(())
        }));

        let gcs = UdpSocket::bind("127.0.0.1:0").await.expect("bind sender");
        let packet = v1_fixture_with_header(
            &request_message(),
            MavHeader {
                sequence: 42,
                system_id: 250,
                component_id: 191,
            },
        );
        gcs.send_to(&packet, gcs_gateway_addr)
            .await
            .expect("send command");

        let mut buffer = vec![0_u8; 2048];
        let (len, _) = timeout(
            Duration::from_secs(1),
            vehicle_receiver.recv_from(&mut buffer),
        )
        .await
        .expect("vehicle receives allowed MAVLink v1 command")
        .expect("recv succeeds");
        assert_eq!(&buffer[..len], packet.as_slice());

        let _ = shutdown_tx.send(());
        task.await.expect("task joins").expect("gateway stops");
    }

    #[tokio::test]
    async fn forwards_signed_vehicle_packet_without_modifying_bytes() {
        let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind GCS");
        let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await.expect("bind vehicle");
        let gateway = UdpGateway::bind(test_config(
            gcs_receiver.local_addr().expect("GCS addr"),
            vehicle_receiver.local_addr().expect("vehicle addr"),
        ))
        .await
        .expect("gateway binds");
        let vehicle_gateway_addr = gateway.vehicle_listen_addr().expect("vehicle listen addr");
        let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(gateway.run_until_shutdown(async {
            let _ = shutdown_rx.await;
            Ok(())
        }));

        let sitl = UdpSocket::bind("127.0.0.1:0").await.expect("bind SITL");
        let packet = signed_fixture_with_header(
            &common::MavMessage::SYS_STATUS(common::SYS_STATUS_DATA::default()),
            MavHeader {
                sequence: 43,
                system_id: 42,
                component_id: 200,
            },
        );
        sitl.send_to(&packet, vehicle_gateway_addr)
            .await
            .expect("send signed packet");

        let mut buffer = vec![0_u8; 2048];
        let (len, _) = timeout(Duration::from_secs(1), gcs_receiver.recv_from(&mut buffer))
            .await
            .expect("GCS receives signed packet")
            .expect("recv succeeds");
        assert_eq!(&buffer[..len], packet.as_slice());

        let _ = shutdown_tx.send(());
        task.await.expect("task joins").expect("gateway stops");
    }

    #[tokio::test]
    async fn metrics_track_parse_processing_policy_and_blocks() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let invalid = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            b"SECRET_PAYLOAD_SHOULD_NOT_BE_LOGGED",
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        assert_eq!(invalid, DatagramOutcome::DropInvalid);

        let forwarded = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &heartbeat(3),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        assert_eq!(
            forwarded,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );

        let blocked = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        assert_eq!(blocked, DatagramOutcome::Blocked);

        let counters = *counters.lock().await;
        assert_eq!(counters.packets_received_total, 3);
        assert_eq!(counters.packets_parse_error_total, 1);
        assert_eq!(counters.packets_signed_observed_total, 0);
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.commands_critical_observed_total, 1);
        assert_eq!(counters.parse_latency_samples, 3);
        assert_eq!(counters.policy_latency_samples, 1);
        assert_eq!(counters.processing_latency_samples, 3);
    }

    #[tokio::test]
    async fn metrics_track_processing_latency_for_forwarded_gcs_non_commands() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let forwarded = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &heartbeat(0),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;

        assert_eq!(
            forwarded,
            DatagramOutcome::Forward {
                target: ForwardTarget::Vehicle
            }
        );

        let counters = *counters.lock().await;
        assert_eq!(counters.packets_received_total, 1);
        assert_eq!(counters.packets_forwarded_total, 0);
        assert_eq!(counters.parse_latency_samples, 1);
        assert_eq!(counters.policy_latency_samples, 0);
        assert_eq!(counters.processing_latency_samples, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn signed_packets_are_observed_without_claiming_authentication() {
        let output = StdArc::new(StdMutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_writer(CapturedWriter::new(StdArc::clone(&output)))
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &signed_heartbeat(3),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        drop(guard);

        assert_eq!(
            outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        assert_eq!(counters.lock().await.packets_signed_observed_total, 1);

        let output = String::from_utf8(output.lock().expect("log buffer lock").clone())
            .expect("logs are UTF-8");
        assert!(output.contains("mavlink.signed_observed"));
        assert!(output.contains("authenticated=false"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn signing_audit_validates_signed_packets_without_blocking() {
        let output = StdArc::new(StdMutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_writer(CapturedWriter::new(StdArc::clone(&output)))
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &signed_heartbeat(3),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Audit,
                validator: Some(&signing_validator),
            },
        )
        .await;
        drop(guard);

        assert_eq!(
            outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_valid_total, 1);
        assert_eq!(counters.packets_signed_invalid_total, 0);

        let output = String::from_utf8(output.lock().expect("log buffer lock").clone())
            .expect("logs are UTF-8");
        assert!(output.contains("mavlink.signing_validated"));
        assert!(output.contains("authenticated=true"));
        assert!(!output.contains("1021324354657687"));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn signing_audit_rejects_bad_signature_but_still_forwards() {
        let output = StdArc::new(StdMutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_writer(CapturedWriter::new(StdArc::clone(&output)))
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &tampered_signature(signed_heartbeat(3)),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Audit,
                validator: Some(&signing_validator),
            },
        )
        .await;
        drop(guard);

        assert_eq!(
            outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_valid_total, 0);
        assert_eq!(counters.packets_signed_invalid_total, 1);

        let output = String::from_utf8(output.lock().expect("log buffer lock").clone())
            .expect("logs are UTF-8");
        assert!(output.contains("mavlink.signing_rejected"));
        assert!(output.contains("authenticated=false"));
        assert!(output.contains("invalid_signature_or_timestamp"));
    }

    #[tokio::test]
    async fn signing_audit_counts_replay_without_blocking() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);
        let packet = signed_heartbeat(3);

        let first = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &packet,
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Audit,
                validator: Some(&signing_validator),
            },
        )
        .await;
        let replay = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &packet,
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Audit,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(
            first,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        assert_eq!(
            replay,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_signed_valid_total, 1);
        assert_eq!(counters.packets_signed_invalid_total, 1);
        assert_eq!(counters.signing_replay_rejected_total, 1);
    }

    #[tokio::test]
    async fn signing_enforce_allows_valid_signed_critical_command_to_semantic_policy() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let heartbeat_outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &heartbeat(3),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;
        assert_eq!(
            heartbeat_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );

        let command_outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &signed_arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(
            command_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Vehicle
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_valid_total, 1);
        assert_eq!(counters.packets_blocked_total, 0);
        assert_eq!(counters.commands_critical_observed_total, 1);
    }

    #[tokio::test]
    async fn signing_enforce_blocks_unsigned_critical_command_before_audit_only() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let mut policy = test_policy();
        policy.audit_only = true;
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_unsigned_rejected_total, 1);
        assert_eq!(counters.packets_signed_observed_total, 0);
        assert_eq!(counters.policy_latency_samples, 0);
    }

    #[tokio::test]
    async fn signing_enforce_blocks_unsigned_mavlink_v1_critical_command() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command_v1(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_unsigned_rejected_total, 1);
        assert_eq!(counters.commands_critical_observed_total, 1);
        assert_eq!(counters.packets_signed_observed_total, 0);
        assert_eq!(counters.policy_latency_samples, 0);
    }

    #[tokio::test]
    async fn signing_enforce_blocks_unsigned_takeoff_before_nav_movement_policy() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &takeoff_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_unsigned_rejected_total, 1);
        assert_eq!(counters.commands_critical_observed_total, 1);
        assert_eq!(counters.policy_latency_samples, 0);
    }

    #[tokio::test]
    async fn signing_enforce_blocks_critical_command_with_bad_signature() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &tampered_signature(signed_arm_command()),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_invalid_total, 1);
        assert_eq!(counters.packets_signed_valid_total, 0);
    }

    #[tokio::test]
    async fn signing_enforce_blocks_critical_command_with_unexpected_link_id() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator =
            SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 42);

        let outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &signed_arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_invalid_total, 1);
        assert_eq!(counters.packets_signed_valid_total, 0);
    }

    #[tokio::test]
    async fn signing_enforce_blocks_critical_command_replay() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);
        let packet = signed_arm_command();

        let first = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &packet,
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;
        let replay = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &packet,
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(first, DatagramOutcome::Blocked);
        assert_eq!(replay, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_signed_valid_total, 1);
        assert_eq!(counters.packets_signed_invalid_total, 1);
        assert_eq!(counters.signing_replay_rejected_total, 1);
        assert_eq!(counters.packets_blocked_total, 2);
    }

    #[tokio::test]
    async fn signing_enforce_keeps_vehicle_telemetry_available() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let unsigned = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &heartbeat(3),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;
        let invalid_signed = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &tampered_signature(signed_heartbeat(3)),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(
            unsigned,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        assert_eq!(
            invalid_signed,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 0);
        assert_eq!(counters.packets_unsigned_rejected_total, 0);
        assert_eq!(counters.packets_signed_observed_total, 1);
        assert_eq!(counters.packets_signed_invalid_total, 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shadow_enforce_reports_policy_would_block_without_blocking() {
        let output = StdArc::new(StdMutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_writer(CapturedWriter::new(StdArc::clone(&output)))
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let mut policy = test_policy();
        policy.audit_only = true;
        policy.shadow_enforce = true;
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let heartbeat_outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &heartbeat(3),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        assert_eq!(
            heartbeat_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );

        let command_outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        drop(guard);

        assert_eq!(
            command_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Vehicle
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 0);
        assert_eq!(counters.shadow_policy_would_block_total, 1);
        assert_eq!(counters.shadow_signing_would_reject_total, 1);
        assert_eq!(counters.shadow_unsigned_critical_total, 1);

        let output = String::from_utf8(output.lock().expect("log buffer lock").clone())
            .expect("logs are UTF-8");
        assert!(output.contains("security.shadow_would_block"));
        assert!(output.contains("signing.shadow_would_reject"));
        assert!(!output.contains("payload"));
        assert!(!output.contains("1021324354657687"));
    }

    #[tokio::test]
    async fn shadow_enforce_reports_unsigned_critical_without_rejecting_packet() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let mut policy = certified_loopback_policy();
        policy.shadow_enforce = true;
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let heartbeat_outcome = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &heartbeat(0),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        assert_eq!(
            heartbeat_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Gcs
            }
        );

        let command_outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;

        assert_eq!(
            command_outcome,
            DatagramOutcome::Forward {
                target: ForwardTarget::Vehicle
            }
        );
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 0);
        assert_eq!(counters.packets_unsigned_rejected_total, 0);
        assert_eq!(counters.shadow_signing_would_reject_total, 1);
        assert_eq!(counters.shadow_unsigned_critical_total, 1);
        assert_eq!(counters.shadow_policy_would_block_total, 0);
    }

    #[tokio::test]
    async fn shadow_enforce_does_not_degrade_real_signing_enforce_block() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let mut policy = certified_loopback_policy();
        policy.audit_only = true;
        policy.shadow_enforce = true;
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = SigningValidator::new(crate::signing::INSECURE_TEST_SIGNING_KEY, 7);

        let outcome = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Enforce,
                validator: Some(&signing_validator),
            },
        )
        .await;

        assert_eq!(outcome, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 1);
        assert_eq!(counters.packets_unsigned_rejected_total, 1);
        assert_eq!(counters.shadow_signing_would_reject_total, 0);
        assert_eq!(counters.policy_latency_samples, 0);
    }

    #[tokio::test]
    async fn setup_signing_is_blocked_in_both_directions_without_payload_logging() {
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = certified_loopback_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let from_gcs = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &setup_signing(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        let from_vehicle = process_datagram(
            UdpFlow::VehicleToGcs,
            source_ip,
            &setup_signing(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;

        assert_eq!(from_gcs, DatagramOutcome::Blocked);
        assert_eq!(from_vehicle, DatagramOutcome::Blocked);
        let counters = *counters.lock().await;
        assert_eq!(counters.packets_blocked_total, 2);
        assert_eq!(counters.packets_forwarded_total, 0);
        assert_eq!(counters.setup_signing_observed_total, 2);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn security_logs_do_not_include_payload_or_secret_material() {
        let output = StdArc::new(StdMutex::new(Vec::new()));
        let subscriber = tracing_subscriber::fmt()
            .with_ansi(false)
            .with_max_level(tracing::Level::TRACE)
            .without_time()
            .with_writer(CapturedWriter::new(StdArc::clone(&output)))
            .finish();
        let guard = tracing::subscriber::set_default(subscriber);
        let flight_state = Mutex::new(FlightState::default());
        let counters = Mutex::new(GatewayCounters::default());
        let policy = test_policy();
        let source_ip = "127.0.0.1".parse().expect("valid IP");
        let signing_validator = no_signing_validator();

        let _ = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            b"SECRET_KEY=abc123;payload=top-secret",
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        let blocked = process_datagram(
            UdpFlow::GcsToVehicle,
            source_ip,
            &arm_command(),
            &flight_state,
            &counters,
            &policy,
            SigningAuditContext {
                policy: SigningPolicy::Observe,
                validator: signing_validator.as_deref(),
            },
        )
        .await;
        drop(guard);

        assert_eq!(blocked, DatagramOutcome::Blocked);
        assert_eq!(counters.lock().await.packets_blocked_total, 1);

        let output = String::from_utf8(output.lock().expect("log buffer lock").clone())
            .expect("logs are UTF-8");
        assert!(output.contains("mavlink.parse_error"));
        assert!(!output.contains("SECRET_KEY"));
        assert!(!output.contains("abc123"));
        assert!(!output.contains("top-secret"));
        assert!(!output.contains("payload=top-secret"));
    }

    #[derive(Clone)]
    struct CapturedWriter {
        buffer: StdArc<StdMutex<Vec<u8>>>,
    }

    impl CapturedWriter {
        fn new(buffer: StdArc<StdMutex<Vec<u8>>>) -> Self {
            Self { buffer }
        }
    }

    impl<'writer> MakeWriter<'writer> for CapturedWriter {
        type Writer = CapturedWriteGuard;

        fn make_writer(&'writer self) -> Self::Writer {
            CapturedWriteGuard {
                buffer: StdArc::clone(&self.buffer),
            }
        }
    }

    struct CapturedWriteGuard {
        buffer: StdArc<StdMutex<Vec<u8>>>,
    }

    impl Write for CapturedWriteGuard {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.buffer
                .lock()
                .expect("log buffer lock")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
}
