use std::io::Cursor;

use mavlink::{
    common, peek_reader::PeekReader, MAVLinkMessageRaw, MavHeader, MavlinkVersion, Message,
};
use thiserror::Error;

use crate::{
    flight_state::{Autopilot, HeartbeatObservation, VehicleFamily},
    mavlink_constants::MAVLINK2_SIGNED_INCOMPAT_FLAG,
    security_filter::{
        CommandMessage, MAVLINK_MSG_ID_COMMAND_INT, MAVLINK_MSG_ID_COMMAND_LONG,
        MAVLINK_MSG_ID_MANUAL_CONTROL, MAVLINK_MSG_ID_MISSION_CLEAR_ALL,
        MAVLINK_MSG_ID_MISSION_COUNT, MAVLINK_MSG_ID_MISSION_ITEM, MAVLINK_MSG_ID_MISSION_ITEM_INT,
        MAVLINK_MSG_ID_MISSION_SET_CURRENT, MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST,
        MAVLINK_MSG_ID_PARAM_SET, MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE, MAVLINK_MSG_ID_SET_MODE,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecStatus {
    Ready,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawFrameInfo {
    pub protocol_version: MavlinkVersion,
    pub sequence: u8,
    pub system_id: u8,
    pub component_id: u8,
    pub message_id: u32,
    pub signed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedMavlinkPacket {
    pub frame: RawFrameInfo,
    pub semantic: MavlinkSemantic,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MavlinkSemantic {
    Heartbeat(HeartbeatObservation),
    Command(CommandMessage),
    SetupSigning,
    Other { message_name: &'static str },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DecodeError {
    #[error("datagram does not start with a MAVLink v1 or v2 marker")]
    InvalidStartMarker,
    #[error("datagram contains trailing bytes after one MAVLink frame")]
    TrailingBytes,
    #[error("MAVLink frame could not be parsed")]
    ParseFailed,
}

pub fn decode_datagram(bytes: &[u8]) -> Result<ParsedMavlinkPacket, DecodeError> {
    match bytes.first().copied() {
        Some(mavlink::MAV_STX | mavlink::MAV_STX_V2) => {}
        _ => return Err(DecodeError::InvalidStartMarker),
    }

    let mut reader = PeekReader::new(Cursor::new(bytes));
    let raw = mavlink::read_any_raw_message::<common::MavMessage, _>(&mut reader)
        .map_err(|_| DecodeError::ParseFailed)?;

    if raw_bytes(&raw).len() != bytes.len() {
        return Err(DecodeError::TrailingBytes);
    }

    let header = MavHeader {
        sequence: raw.sequence(),
        system_id: raw.system_id(),
        component_id: raw.component_id(),
    };
    let message = common::MavMessage::parse(raw.version(), raw.message_id(), raw.payload())
        .map_err(|_| DecodeError::ParseFailed)?;

    Ok(ParsedMavlinkPacket {
        frame: RawFrameInfo {
            protocol_version: raw.version(),
            sequence: header.sequence,
            system_id: header.system_id,
            component_id: header.component_id,
            message_id: raw.message_id(),
            signed: is_signed(&raw),
        },
        semantic: semantic_from_message(header, message),
    })
}

pub fn parse_error_decision() -> crate::security_filter::PolicyDecision {
    crate::security_filter::evaluate_parse_error()
}

fn raw_bytes(raw: &MAVLinkMessageRaw) -> &[u8] {
    match raw {
        MAVLinkMessageRaw::V1(message) => message.raw_bytes(),
        MAVLinkMessageRaw::V2(message) => message.raw_bytes(),
    }
}

fn is_signed(raw: &MAVLinkMessageRaw) -> bool {
    match raw {
        MAVLinkMessageRaw::V1(_) => false,
        MAVLinkMessageRaw::V2(message) => {
            message.incompatibility_flags() & MAVLINK2_SIGNED_INCOMPAT_FLAG != 0
        }
    }
}

#[allow(deprecated)]
fn semantic_from_message(header: MavHeader, message: common::MavMessage) -> MavlinkSemantic {
    match message {
        common::MavMessage::HEARTBEAT(heartbeat) => {
            MavlinkSemantic::Heartbeat(heartbeat_observation(header, heartbeat))
        }
        common::MavMessage::COMMAND_LONG(command) => {
            MavlinkSemantic::Command(command_long_message(command))
        }
        common::MavMessage::COMMAND_INT(command) => {
            MavlinkSemantic::Command(command_int_message(command))
        }
        common::MavMessage::SET_MODE(_) => {
            MavlinkSemantic::Command(CommandMessage::mavlink_message(MAVLINK_MSG_ID_SET_MODE))
        }
        common::MavMessage::PARAM_SET(_) => {
            MavlinkSemantic::Command(CommandMessage::mavlink_message(MAVLINK_MSG_ID_PARAM_SET))
        }
        common::MavMessage::MISSION_WRITE_PARTIAL_LIST(_) => MavlinkSemantic::Command(
            CommandMessage::mavlink_message(MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST),
        ),
        common::MavMessage::MISSION_ITEM(mission_item) => {
            MavlinkSemantic::Command(mission_item_message(mission_item))
        }
        common::MavMessage::MISSION_SET_CURRENT(_) => MavlinkSemantic::Command(
            CommandMessage::mavlink_message(MAVLINK_MSG_ID_MISSION_SET_CURRENT),
        ),
        common::MavMessage::MISSION_COUNT(_) => MavlinkSemantic::Command(
            CommandMessage::mavlink_message(MAVLINK_MSG_ID_MISSION_COUNT),
        ),
        common::MavMessage::MISSION_CLEAR_ALL(_) => MavlinkSemantic::Command(
            CommandMessage::mavlink_message(MAVLINK_MSG_ID_MISSION_CLEAR_ALL),
        ),
        common::MavMessage::MISSION_ITEM_INT(mission_item) => {
            MavlinkSemantic::Command(mission_item_int_message(mission_item))
        }
        common::MavMessage::MANUAL_CONTROL(_) => MavlinkSemantic::Command(
            CommandMessage::mavlink_message(MAVLINK_MSG_ID_MANUAL_CONTROL),
        ),
        common::MavMessage::RC_CHANNELS_OVERRIDE(_) => MavlinkSemantic::Command(
            CommandMessage::mavlink_message(MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE),
        ),
        common::MavMessage::SETUP_SIGNING(_) => MavlinkSemantic::SetupSigning,
        other => MavlinkSemantic::Other {
            message_name: other.message_name(),
        },
    }
}

fn heartbeat_observation(
    header: MavHeader,
    heartbeat: common::HEARTBEAT_DATA,
) -> HeartbeatObservation {
    HeartbeatObservation {
        autopilot: autopilot(heartbeat.autopilot),
        vehicle_family: vehicle_family(heartbeat.mavtype),
        base_mode: heartbeat.base_mode.bits(),
        custom_mode: Some(heartbeat.custom_mode),
        source_system: header.system_id,
        source_component: header.component_id,
    }
}

fn command_long_message(command: common::COMMAND_LONG_DATA) -> CommandMessage {
    CommandMessage {
        message_id: MAVLINK_MSG_ID_COMMAND_LONG,
        command: command.command as u16,
        param1: command.param1,
        param2: command.param2,
    }
}

fn command_int_message(command: common::COMMAND_INT_DATA) -> CommandMessage {
    CommandMessage {
        message_id: MAVLINK_MSG_ID_COMMAND_INT,
        command: command.command as u16,
        param1: command.param1,
        param2: command.param2,
    }
}

#[allow(deprecated)]
fn mission_item_message(mission_item: common::MISSION_ITEM_DATA) -> CommandMessage {
    CommandMessage {
        message_id: MAVLINK_MSG_ID_MISSION_ITEM,
        command: mission_item.command as u16,
        param1: mission_item.param1,
        param2: mission_item.param2,
    }
}

fn mission_item_int_message(mission_item: common::MISSION_ITEM_INT_DATA) -> CommandMessage {
    CommandMessage {
        message_id: MAVLINK_MSG_ID_MISSION_ITEM_INT,
        command: mission_item.command as u16,
        param1: mission_item.param1,
        param2: mission_item.param2,
    }
}

fn autopilot(autopilot: common::MavAutopilot) -> Autopilot {
    match autopilot {
        common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA => Autopilot::ArduPilot,
        common::MavAutopilot::MAV_AUTOPILOT_PX4 => Autopilot::Px4,
        _ => Autopilot::Unknown,
    }
}

fn vehicle_family(mavtype: common::MavType) -> VehicleFamily {
    match mavtype {
        common::MavType::MAV_TYPE_QUADROTOR
        | common::MavType::MAV_TYPE_COAXIAL
        | common::MavType::MAV_TYPE_HELICOPTER
        | common::MavType::MAV_TYPE_HEXAROTOR
        | common::MavType::MAV_TYPE_OCTOROTOR
        | common::MavType::MAV_TYPE_TRICOPTER => VehicleFamily::ArduCopter,
        common::MavType::MAV_TYPE_FIXED_WING => VehicleFamily::ArduPlane,
        common::MavType::MAV_TYPE_GROUND_ROVER => VehicleFamily::Rover,
        _ => VehicleFamily::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        flight_state::{FlightModeClassification, FlightState},
        security_filter::{
            evaluate_command, Direction, MessageContext, PolicyDecision, SecurityPolicy,
            TransportKind, MAVLINK_MSG_ID_MANUAL_CONTROL, MAVLINK_MSG_ID_MISSION_CLEAR_ALL,
            MAVLINK_MSG_ID_MISSION_COUNT, MAVLINK_MSG_ID_MISSION_ITEM,
            MAVLINK_MSG_ID_MISSION_ITEM_INT, MAVLINK_MSG_ID_MISSION_SET_CURRENT,
            MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST, MAVLINK_MSG_ID_PARAM_SET,
            MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE, MAVLINK_MSG_ID_SET_MODE,
            MAV_CMD_COMPONENT_ARM_DISARM, MAV_CMD_NAV_LAND, MAV_CMD_NAV_TAKEOFF,
            MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN,
        },
    };

    fn header() -> MavHeader {
        MavHeader {
            sequence: 7,
            system_id: 1,
            component_id: 1,
        }
    }

    fn fixture(message: &common::MavMessage) -> Vec<u8> {
        fixture_with_header(message, header())
    }

    fn fixture_with_header(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v2_msg(&mut bytes, header, message).expect("fixture serializes");
        bytes
    }

    fn v1_fixture_with_header(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v1_msg(&mut bytes, header, message).expect("v1 fixture serializes");
        bytes
    }

    fn signed_fixture(message: &common::MavMessage) -> Vec<u8> {
        let mut raw = mavlink::MAVLinkV2MessageRaw::new();
        raw.serialize_message_for_signing(header(), message);
        raw.raw_bytes().to_vec()
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

    fn command_long(command: common::MavCmd, param1: f32, param2: f32) -> Vec<u8> {
        fixture(&common::MavMessage::COMMAND_LONG(
            common::COMMAND_LONG_DATA {
                param1,
                param2,
                param3: 0.0,
                param4: 0.0,
                param5: 0.0,
                param6: 0.0,
                param7: 0.0,
                command,
                target_system: 1,
                target_component: 1,
                confirmation: 0,
            },
        ))
    }

    fn command_int(command: common::MavCmd, param1: f32, param2: f32) -> Vec<u8> {
        fixture(&common::MavMessage::COMMAND_INT(common::COMMAND_INT_DATA {
            param1,
            param2,
            param3: 0.0,
            param4: 0.0,
            x: 0,
            y: 0,
            z: 0.0,
            command,
            target_system: 1,
            target_component: 1,
            frame: common::MavFrame::MAV_FRAME_GLOBAL,
            current: 0,
            autocontinue: 0,
        }))
    }

    fn default_message_fixture(message: common::MavMessage) -> Vec<u8> {
        fixture(&message)
    }

    #[test]
    fn parses_heartbeat_fixture_into_flight_state_observation() {
        let packet = decode_datagram(&heartbeat(3)).expect("valid heartbeat");

        assert_eq!(packet.frame.protocol_version, MavlinkVersion::V2);
        assert_eq!(packet.frame.message_id, 0);
        assert!(!packet.frame.signed);

        let MavlinkSemantic::Heartbeat(observation) = packet.semantic else {
            panic!("expected heartbeat");
        };
        assert_eq!(observation.autopilot, Autopilot::ArduPilot);
        assert_eq!(observation.vehicle_family, VehicleFamily::ArduCopter);
        assert_eq!(observation.custom_mode, Some(3));
        assert_eq!(observation.classify(), FlightModeClassification::Automatic);
    }

    #[test]
    fn parses_mavlink_v1_with_routing_fields() {
        let header = MavHeader {
            sequence: 99,
            system_id: 42,
            component_id: 191,
        };
        let bytes = v1_fixture_with_header(
            &common::MavMessage::SYS_STATUS(common::SYS_STATUS_DATA::default()),
            header,
        );

        let packet = decode_datagram(&bytes).expect("valid MAVLink v1 packet");

        assert_eq!(packet.frame.protocol_version, MavlinkVersion::V1);
        assert_eq!(packet.frame.sequence, header.sequence);
        assert_eq!(packet.frame.system_id, header.system_id);
        assert_eq!(packet.frame.component_id, header.component_id);
        assert_eq!(packet.frame.message_id, 1);
        assert!(!packet.frame.signed);
    }

    #[test]
    fn heartbeat_fixture_updates_flight_state() {
        let packet = decode_datagram(&heartbeat(5)).expect("valid heartbeat");
        let MavlinkSemantic::Heartbeat(observation) = packet.semantic else {
            panic!("expected heartbeat");
        };

        let mut state = FlightState::default();
        let event = state
            .update_heartbeat(observation)
            .expect("first heartbeat changes state");

        assert_eq!(event.classification, FlightModeClassification::NotAutomatic);
        assert_eq!(event.mode_name, Some("Loiter"));
    }

    #[test]
    fn parses_px4_heartbeat_as_limited_unknown_mode() {
        let packet = decode_datagram(&px4_heartbeat(4)).expect("valid PX4 heartbeat");

        let MavlinkSemantic::Heartbeat(observation) = packet.semantic else {
            panic!("expected heartbeat");
        };
        assert_eq!(observation.autopilot, Autopilot::Px4);
        assert_eq!(observation.vehicle_family, VehicleFamily::ArduCopter);
        assert_eq!(observation.custom_mode, Some(4));
        assert_eq!(observation.classify(), FlightModeClassification::Unknown);

        let mut state = FlightState::default();
        let event = state
            .update_heartbeat(observation)
            .expect("first PX4 heartbeat changes state");
        assert_eq!(event.mode_name, None);
        assert_eq!(event.classification, FlightModeClassification::Unknown);
    }

    #[test]
    fn parses_command_long_fixture_into_security_command() {
        let packet = decode_datagram(&command_long(
            common::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
            1.0,
            0.0,
        ))
        .expect("valid command_long");

        assert_eq!(packet.frame.message_id, MAVLINK_MSG_ID_COMMAND_LONG);
        assert_eq!(
            packet.semantic,
            MavlinkSemantic::Command(CommandMessage {
                message_id: MAVLINK_MSG_ID_COMMAND_LONG,
                command: MAV_CMD_COMPONENT_ARM_DISARM,
                param1: 1.0,
                param2: 0.0,
            })
        );
    }

    #[test]
    fn parses_command_int_fixture_into_security_command() {
        let packet = decode_datagram(&command_int(common::MavCmd::MAV_CMD_NAV_TAKEOFF, 0.0, 0.0))
            .expect("valid command_int");

        assert_eq!(
            packet.semantic,
            MavlinkSemantic::Command(CommandMessage {
                message_id: MAVLINK_MSG_ID_COMMAND_INT,
                command: MAV_CMD_NAV_TAKEOFF,
                param1: 0.0,
                param2: 0.0,
            })
        );
    }

    #[test]
    fn parses_nav_land_command_long_fixture_into_security_command() {
        let packet = decode_datagram(&command_long(common::MavCmd::MAV_CMD_NAV_LAND, 0.0, 0.0))
            .expect("valid command_long");

        assert_eq!(
            packet.semantic,
            MavlinkSemantic::Command(CommandMessage {
                message_id: MAVLINK_MSG_ID_COMMAND_LONG,
                command: MAV_CMD_NAV_LAND,
                param1: 0.0,
                param2: 0.0,
            })
        );
    }

    #[test]
    fn parses_preflight_reboot_command_long_fixture_into_security_command() {
        let packet = decode_datagram(&command_long(
            common::MavCmd::MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN,
            0.0,
            0.0,
        ))
        .expect("valid command_long");

        assert_eq!(
            packet.semantic,
            MavlinkSemantic::Command(CommandMessage {
                message_id: MAVLINK_MSG_ID_COMMAND_LONG,
                command: MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN,
                param1: 0.0,
                param2: 0.0,
            })
        );
    }

    #[test]
    #[allow(deprecated)]
    fn parses_critical_message_fixtures_into_security_commands() {
        let fixtures = [
            (
                default_message_fixture(common::MavMessage::SET_MODE(
                    common::SET_MODE_DATA::default(),
                )),
                MAVLINK_MSG_ID_SET_MODE,
            ),
            (
                default_message_fixture(common::MavMessage::PARAM_SET(
                    common::PARAM_SET_DATA::default(),
                )),
                MAVLINK_MSG_ID_PARAM_SET,
            ),
            (
                default_message_fixture(common::MavMessage::MISSION_COUNT(
                    common::MISSION_COUNT_DATA::default(),
                )),
                MAVLINK_MSG_ID_MISSION_COUNT,
            ),
            (
                default_message_fixture(common::MavMessage::MISSION_ITEM(
                    common::MISSION_ITEM_DATA::default(),
                )),
                MAVLINK_MSG_ID_MISSION_ITEM,
            ),
            (
                default_message_fixture(common::MavMessage::MISSION_ITEM_INT(
                    common::MISSION_ITEM_INT_DATA::default(),
                )),
                MAVLINK_MSG_ID_MISSION_ITEM_INT,
            ),
            (
                default_message_fixture(common::MavMessage::MISSION_CLEAR_ALL(
                    common::MISSION_CLEAR_ALL_DATA::default(),
                )),
                MAVLINK_MSG_ID_MISSION_CLEAR_ALL,
            ),
            (
                default_message_fixture(common::MavMessage::MISSION_SET_CURRENT(
                    common::MISSION_SET_CURRENT_DATA::default(),
                )),
                MAVLINK_MSG_ID_MISSION_SET_CURRENT,
            ),
            (
                default_message_fixture(common::MavMessage::MISSION_WRITE_PARTIAL_LIST(
                    common::MISSION_WRITE_PARTIAL_LIST_DATA::default(),
                )),
                MAVLINK_MSG_ID_MISSION_WRITE_PARTIAL_LIST,
            ),
            (
                default_message_fixture(common::MavMessage::MANUAL_CONTROL(
                    common::MANUAL_CONTROL_DATA::default(),
                )),
                MAVLINK_MSG_ID_MANUAL_CONTROL,
            ),
            (
                default_message_fixture(common::MavMessage::RC_CHANNELS_OVERRIDE(
                    common::RC_CHANNELS_OVERRIDE_DATA::default(),
                )),
                MAVLINK_MSG_ID_RC_CHANNELS_OVERRIDE,
            ),
        ];

        for (bytes, expected_message_id) in fixtures {
            let packet = decode_datagram(&bytes).expect("valid critical message fixture");
            assert_eq!(packet.frame.message_id, expected_message_id);
            let MavlinkSemantic::Command(command) = packet.semantic else {
                panic!("expected security command");
            };
            assert_eq!(command.message_id, expected_message_id);
        }
    }

    #[test]
    fn parsed_command_connects_to_security_policy() {
        let packet = decode_datagram(&command_long(
            common::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM,
            1.0,
            0.0,
        ))
        .expect("valid command_long");
        let MavlinkSemantic::Command(command) = packet.semantic else {
            panic!("expected command");
        };
        let policy = SecurityPolicy::new(Vec::new());
        let context = MessageContext {
            direction: Direction::GcsToVehicle,
            transport: TransportKind::Udp,
            source_ip: None,
            flight_mode: FlightModeClassification::Automatic,
        };

        assert!(matches!(
            evaluate_command(&policy, &context, &command),
            PolicyDecision::Block(_)
        ));
    }

    #[test]
    fn observes_mavlink2_signing_flag_without_claiming_authentication() {
        let message = common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode: 3,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        });

        let packet = decode_datagram(&signed_fixture(&message)).expect("signed frame parses");

        assert!(packet.frame.signed);
    }

    #[test]
    fn invalid_packet_maps_to_drop_invalid() {
        let mut invalid = heartbeat(3);
        let last = invalid.last_mut().expect("fixture is non-empty");
        *last = last.wrapping_add(1);

        assert_eq!(decode_datagram(&invalid), Err(DecodeError::ParseFailed));
        assert!(matches!(
            parse_error_decision(),
            PolicyDecision::DropInvalid(_)
        ));
    }

    #[test]
    fn packet_with_trailing_bytes_is_invalid_for_udp_datagram_boundary() {
        let mut bytes = heartbeat(3);
        bytes.push(0);

        assert_eq!(decode_datagram(&bytes), Err(DecodeError::TrailingBytes));
    }

    #[test]
    fn truncated_packet_is_invalid_for_udp_datagram_boundary() {
        let mut bytes = heartbeat(3);
        bytes.truncate(bytes.len().saturating_sub(2));

        assert_eq!(decode_datagram(&bytes), Err(DecodeError::ParseFailed));
    }

    #[test]
    fn setup_signing_is_classified_as_sensitive_semantic() {
        let bytes = fixture(&common::MavMessage::SETUP_SIGNING(
            common::SETUP_SIGNING_DATA::default(),
        ));

        let packet = decode_datagram(&bytes).expect("setup signing fixture parses");

        assert_eq!(packet.frame.message_id, 256);
        assert_eq!(packet.semantic, MavlinkSemantic::SetupSigning);
    }
}
