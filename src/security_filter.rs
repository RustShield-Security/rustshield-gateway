use std::net::IpAddr;

use crate::flight_state::FlightModeClassification;

pub const MAVLINK_MSG_ID_COMMAND_LONG: u32 = 76;
pub const MAVLINK_MSG_ID_COMMAND_INT: u32 = 75;
pub const MAVLINK_MSG_ID_SET_MODE: u32 = 11;
pub const MAVLINK_MSG_ID_PARAM_SET: u32 = 23;
pub const MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST: u32 = 38;
pub const MAVLINK_MSG_ID_MISSION_ITEM: u32 = 39;
pub const MAVLINK_MSG_ID_MISSION_SET_CURRENT: u32 = 41;
pub const MAVLINK_MSG_ID_MISSION_COUNT: u32 = 44;
pub const MAVLINK_MSG_ID_MISSION_CLEAR_ALL: u32 = 45;
pub const MAVLINK_MSG_ID_MISSION_ITEM_INT: u32 = 73;
pub const MAVLINK_MSG_ID_MANUAL_CONTROL: u32 = 69;
pub const MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE: u32 = 70;
pub const MAV_CMD_COMPONENT_ARM_DISARM: u16 = 400;
pub const MAV_CMD_NAV_TAKEOFF: u16 = 22;
pub const MAV_CMD_NAV_LAND: u16 = 21;
pub const MAV_CMD_DO_SET_MODE: u16 = 176;
pub const MAV_CMD_MISSION_START: u16 = 300;
pub const MAV_CMD_DO_REPOSITION: u16 = 192;
pub const MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN: u16 = 246;
pub const FORCE_ARM_DISARM_MAGIC: f32 = 21196.0;
pub const NO_MAV_CMD: u16 = 0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    GcsToVehicle,
    VehicleToGcs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Udp,
    Serial,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageContext {
    pub direction: Direction,
    pub transport: TransportKind,
    pub source_ip: Option<IpAddr>,
    pub flight_mode: FlightModeClassification,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandMessage {
    pub message_id: u32,
    pub command: u16,
    pub param1: f32,
    pub param2: f32,
}

impl CommandMessage {
    pub fn mavlink_message(message_id: u32) -> Self {
        Self {
            message_id,
            command: NO_MAV_CMD,
            param1: 0.0,
            param2: 0.0,
        }
    }

    pub fn is_component_arm_disarm(&self) -> bool {
        matches!(
            self.message_id,
            MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
        ) && self.command == MAV_CMD_COMPONENT_ARM_DISARM
    }

    pub fn is_arm_attempt(&self) -> bool {
        self.is_component_arm_disarm() && float_param_eq(self.param1, 1.0)
    }

    pub fn is_force_arm_attempt(&self) -> bool {
        self.is_arm_attempt() && float_param_eq(self.param2, FORCE_ARM_DISARM_MAGIC)
    }

    pub fn is_cataloged_critical_command(&self) -> bool {
        matches!(self.risk_category(), Some(RiskCategory::Critical))
    }

    pub fn is_cataloged_high_risk_command(&self) -> bool {
        matches!(self.risk_category(), Some(RiskCategory::High))
    }

    pub fn requires_known_flight_mode(&self) -> bool {
        self.risk_category().is_some()
    }

    fn risk_category(&self) -> Option<RiskCategory> {
        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_SET_MODE
                | MAVLINK_MSG_ID_MANUAL_CONTROL
                | MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE
        ) {
            return Some(RiskCategory::Critical);
        }

        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_PARAM_SET
                | MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST
                | MAVLINK_MSG_ID_MISSION_ITEM
                | MAVLINK_MSG_ID_MISSION_SET_CURRENT
                | MAVLINK_MSG_ID_MISSION_COUNT
                | MAVLINK_MSG_ID_MISSION_CLEAR_ALL
                | MAVLINK_MSG_ID_MISSION_ITEM_INT
        ) {
            return Some(RiskCategory::High);
        }

        if !matches!(
            self.message_id,
            MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
        ) {
            return None;
        }

        match self.command {
            MAV_CMD_COMPONENT_ARM_DISARM
            | MAV_CMD_NAV_TAKEOFF
            | MAV_CMD_NAV_LAND
            | MAV_CMD_DO_SET_MODE
            | MAV_CMD_DO_REPOSITION => Some(RiskCategory::Critical),
            MAV_CMD_MISSION_START | MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN => Some(RiskCategory::High),
            _ => None,
        }
    }

    fn explicit_rule_id(&self) -> Option<RuleId> {
        if self.message_id == MAVLINK_MSG_ID_PARAM_SET {
            return Some(RuleId::ParamSet001);
        }

        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST
                | MAVLINK_MSG_ID_MISSION_ITEM
                | MAVLINK_MSG_ID_MISSION_SET_CURRENT
                | MAVLINK_MSG_ID_MISSION_COUNT
                | MAVLINK_MSG_ID_MISSION_CLEAR_ALL
                | MAVLINK_MSG_ID_MISSION_ITEM_INT
        ) || (matches!(
            self.message_id,
            MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
        ) && self.command == MAV_CMD_MISSION_START)
        {
            return Some(RuleId::MissionUpload001);
        }

        if self.message_id == MAVLINK_MSG_ID_SET_MODE
            || (matches!(
                self.message_id,
                MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
            ) && self.command == MAV_CMD_DO_SET_MODE)
        {
            return Some(RuleId::ModeChange001);
        }

        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
        ) && matches!(self.command, MAV_CMD_NAV_TAKEOFF | MAV_CMD_NAV_LAND)
        {
            return Some(RuleId::NavMovement001);
        }

        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
        ) && self.command == MAV_CMD_DO_REPOSITION
        {
            return Some(RuleId::GuidedReposition001);
        }

        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_MANUAL_CONTROL | MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE
        ) {
            return Some(RuleId::RcOverride001);
        }

        if matches!(
            self.message_id,
            MAVLINK_MSG_ID_COMMAND_LONG | MAVLINK_MSG_ID_COMMAND_INT
        ) && self.command == MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN
        {
            return Some(RuleId::PreflightReboot001);
        }

        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RiskCategory {
    Critical,
    High,
}

impl RiskCategory {
    fn severity(self) -> Severity {
        match self {
            Self::Critical => Severity::Critical,
            Self::High => Severity::High,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecurityPolicy {
    pub certified_ips: Vec<IpAddr>,
    pub audit_only: bool,
    pub shadow_enforce: bool,
    pub block_arm_in_auto_mode: bool,
    pub block_critical_when_mode_unknown: bool,
}

impl SecurityPolicy {
    pub fn new(certified_ips: Vec<IpAddr>) -> Self {
        Self {
            certified_ips,
            audit_only: false,
            shadow_enforce: false,
            block_arm_in_auto_mode: true,
            block_critical_when_mode_unknown: true,
        }
    }

    pub fn is_certified_ip(&self, ip: Option<IpAddr>) -> bool {
        ip.is_some_and(|ip| self.certified_ips.contains(&ip))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    Allow,
    Block(BlockReason),
    AuditOnly(BlockReason),
    DropInvalid(InvalidPacketReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockReason {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub param2_class: Param2Class,
    pub audit_event: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleId {
    ArmAuto001,
    CriticalUnknown001,
    ParseError001,
    ParamSet001,
    MissionUpload001,
    ModeChange001,
    NavMovement001,
    GuidedReposition001,
    RcOverride001,
    PreflightReboot001,
}

impl RuleId {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ArmAuto001 => "ARM-AUTO-001",
            Self::CriticalUnknown001 => "CRITICAL-UNKNOWN-001",
            Self::ParseError001 => "PARSE-ERROR-001",
            Self::ParamSet001 => "PARAM-SET-001",
            Self::MissionUpload001 => "MISSION-UPLOAD-001",
            Self::ModeChange001 => "MODE-CHANGE-001",
            Self::NavMovement001 => "NAV-MOVEMENT-001",
            Self::GuidedReposition001 => "GUIDED-REPOSITION-001",
            Self::RcOverride001 => "RC-OVERRIDE-001",
            Self::PreflightReboot001 => "PREFLIGHT-REBOOT-001",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Param2Class {
    Normal,
    Force,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidPacketReason {
    pub rule_id: RuleId,
    pub severity: Severity,
    pub audit_event: &'static str,
    pub reason: &'static str,
}

pub fn evaluate_command(
    policy: &SecurityPolicy,
    context: &MessageContext,
    command: &CommandMessage,
) -> PolicyDecision {
    if context.direction != Direction::GcsToVehicle {
        return PolicyDecision::Allow;
    }

    match context.flight_mode {
        // MVP 0.1 scope: ARM-AUTO-001 only blocks arming in Automatic mode.
        // Known non-automatic modes (e.g. Guided/RTL) are intentionally left
        // to other explicit rules and future policy phases.
        FlightModeClassification::Automatic
            if command.is_arm_attempt()
                && policy.block_arm_in_auto_mode
                && !policy.is_certified_ip(context.source_ip) =>
        {
            deny(policy, RuleId::ArmAuto001, command)
        }
        FlightModeClassification::Unknown
            if policy.block_critical_when_mode_unknown && command.requires_known_flight_mode() =>
        {
            deny(policy, RuleId::CriticalUnknown001, command)
        }
        _ if !policy.is_certified_ip(context.source_ip) => {
            if let Some(rule_id) = command.explicit_rule_id() {
                deny(policy, rule_id, command)
            } else {
                PolicyDecision::Allow
            }
        }
        _ => PolicyDecision::Allow,
    }
}

pub fn evaluate_parse_error() -> PolicyDecision {
    PolicyDecision::DropInvalid(InvalidPacketReason {
        rule_id: RuleId::ParseError001,
        severity: Severity::Medium,
        audit_event: "mavlink.parse_error",
        reason: "packet could not be parsed safely",
    })
}

fn deny(policy: &SecurityPolicy, rule_id: RuleId, command: &CommandMessage) -> PolicyDecision {
    let param2_class = if command.is_force_arm_attempt() {
        Param2Class::Force
    } else if command.is_component_arm_disarm() {
        Param2Class::Normal
    } else {
        Param2Class::NotApplicable
    };
    let severity = if param2_class == Param2Class::Force {
        Severity::Critical
    } else {
        command
            .risk_category()
            .map_or(Severity::Medium, RiskCategory::severity)
    };

    let reason = BlockReason {
        rule_id,
        severity,
        param2_class,
        audit_event: "security.command_blocked",
    };

    if policy.audit_only {
        PolicyDecision::AuditOnly(reason)
    } else {
        PolicyDecision::Block(reason)
    }
}

fn float_param_eq(left: f32, right: f32) -> bool {
    (left - right).abs() <= f32::EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> SecurityPolicy {
        SecurityPolicy::new(vec!["127.0.0.1".parse().expect("valid IP")])
    }

    fn context(source_ip: &str, flight_mode: FlightModeClassification) -> MessageContext {
        MessageContext {
            direction: Direction::GcsToVehicle,
            transport: TransportKind::Udp,
            source_ip: Some(source_ip.parse().expect("valid IP")),
            flight_mode,
        }
    }

    fn serial_context(flight_mode: FlightModeClassification) -> MessageContext {
        MessageContext {
            direction: Direction::GcsToVehicle,
            transport: TransportKind::Serial,
            source_ip: None,
            flight_mode,
        }
    }

    fn arm_command(param2: f32) -> CommandMessage {
        CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: MAV_CMD_COMPONENT_ARM_DISARM,
            param1: 1.0,
            param2,
        }
    }

    #[test]
    fn arm_auto_u_001_blocks_non_certified_arm_in_auto() {
        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::Automatic),
            &arm_command(0.0),
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::ArmAuto001,
                severity: Severity::Critical,
                param2_class: Param2Class::Normal,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn arm_auto_u_002_allows_certified_arm_in_auto() {
        let decision = evaluate_command(
            &policy(),
            &context("127.0.0.1", FlightModeClassification::Automatic),
            &arm_command(0.0),
        );

        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn arm_auto_u_003_blocks_force_arm_as_high_severity() {
        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::Automatic),
            &arm_command(FORCE_ARM_DISARM_MAGIC),
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::ArmAuto001,
                severity: Severity::Critical,
                param2_class: Param2Class::Force,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn arm_auto_u_004_blocks_arm_when_mode_unknown() {
        let decision = evaluate_command(
            &policy(),
            &context("127.0.0.1", FlightModeClassification::Unknown),
            &arm_command(0.0),
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::CriticalUnknown001,
                severity: Severity::Critical,
                param2_class: Param2Class::Normal,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn arm_auto_u_005_allows_non_critical_command() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: 512,
            param1: 1.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::Automatic),
            &command,
        );

        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn arm_auto_u_006_drops_parse_errors() {
        assert_eq!(
            evaluate_parse_error(),
            PolicyDecision::DropInvalid(InvalidPacketReason {
                rule_id: RuleId::ParseError001,
                severity: Severity::Medium,
                audit_event: "mavlink.parse_error",
                reason: "packet could not be parsed safely",
            })
        );
    }

    #[test]
    fn arm_auto_u_007_allows_non_certified_arm_in_known_not_automatic_mode_by_mvp_design() {
        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &arm_command(0.0),
        );

        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn critical_unknown_u_001_blocks_cataloged_critical_command_when_mode_unknown() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: MAV_CMD_NAV_TAKEOFF,
            param1: 0.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("127.0.0.1", FlightModeClassification::Unknown),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::CriticalUnknown001,
                severity: Severity::Critical,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn critical_unknown_blocks_cataloged_message_when_mode_unknown() {
        let command = CommandMessage::mavlink_message(MAVLINK_MSG_ID_PARAM_SET);

        let decision = evaluate_command(
            &policy(),
            &context("127.0.0.1", FlightModeClassification::Unknown),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::CriticalUnknown001,
                severity: Severity::High,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn param_set_001_blocks_non_certified_parameter_mutation() {
        let command = CommandMessage::mavlink_message(MAVLINK_MSG_ID_PARAM_SET);

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::ParamSet001,
                severity: Severity::High,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn mode_change_001_allows_certified_mode_change_when_mode_known() {
        let command = CommandMessage::mavlink_message(MAVLINK_MSG_ID_SET_MODE);

        let decision = evaluate_command(
            &policy(),
            &context("127.0.0.1", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(decision, PolicyDecision::Allow);
    }

    #[test]
    fn mission_upload_001_blocks_mission_messages_from_non_certified_ip() {
        let command = CommandMessage::mavlink_message(MAVLINK_MSG_ID_MISSION_ITEM_INT);

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::MissionUpload001,
                severity: Severity::High,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn guided_reposition_001_blocks_non_certified_reposition_command() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_INT,
            command: MAV_CMD_DO_REPOSITION,
            param1: 0.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::GuidedReposition001,
                severity: Severity::Critical,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn nav_movement_001_blocks_non_certified_takeoff_when_mode_known() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: MAV_CMD_NAV_TAKEOFF,
            param1: 0.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::NavMovement001,
                severity: Severity::Critical,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn nav_movement_001_blocks_non_certified_land_when_mode_known() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: MAV_CMD_NAV_LAND,
            param1: 0.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::NavMovement001,
                severity: Severity::Critical,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn nav_movement_001_allows_certified_takeoff_and_land_when_mode_known() {
        for command in [MAV_CMD_NAV_TAKEOFF, MAV_CMD_NAV_LAND] {
            let decision = evaluate_command(
                &policy(),
                &context("127.0.0.1", FlightModeClassification::NotAutomatic),
                &CommandMessage {
                    message_id: MAVLINK_MSG_ID_COMMAND_INT,
                    command,
                    param1: 0.0,
                    param2: 0.0,
                },
            );

            assert_eq!(decision, PolicyDecision::Allow);
        }
    }

    #[test]
    fn critical_unknown_001_precedes_nav_movement_001_when_mode_unknown() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: MAV_CMD_NAV_LAND,
            param1: 0.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::Unknown),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::CriticalUnknown001,
                severity: Severity::Critical,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn rc_override_001_blocks_manual_control_from_non_certified_ip() {
        let command = CommandMessage::mavlink_message(MAVLINK_MSG_ID_MANUAL_CONTROL);

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::RcOverride001,
                severity: Severity::Critical,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn preflight_reboot_001_blocks_non_certified_reboot_command() {
        let command = CommandMessage {
            message_id: MAVLINK_MSG_ID_COMMAND_LONG,
            command: MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN,
            param1: 0.0,
            param2: 0.0,
        };

        let decision = evaluate_command(
            &policy(),
            &context("192.0.2.10", FlightModeClassification::NotAutomatic),
            &command,
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::PreflightReboot001,
                severity: Severity::High,
                param2_class: Param2Class::NotApplicable,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn audit_only_reports_would_block_without_blocking() {
        let mut policy = policy();
        policy.audit_only = true;

        let decision = evaluate_command(
            &policy,
            &context("192.0.2.10", FlightModeClassification::Automatic),
            &arm_command(0.0),
        );

        assert_eq!(
            decision,
            PolicyDecision::AuditOnly(BlockReason {
                rule_id: RuleId::ArmAuto001,
                severity: Severity::Critical,
                param2_class: Param2Class::Normal,
                audit_event: "security.command_blocked",
            })
        );
    }

    #[test]
    fn serial_transport_applies_same_semantic_policy_without_certified_ip() {
        let decision = evaluate_command(
            &policy(),
            &serial_context(FlightModeClassification::Automatic),
            &arm_command(0.0),
        );

        assert_eq!(
            decision,
            PolicyDecision::Block(BlockReason {
                rule_id: RuleId::ArmAuto001,
                severity: Severity::Critical,
                param2_class: Param2Class::Normal,
                audit_event: "security.command_blocked",
            })
        );
    }
}
