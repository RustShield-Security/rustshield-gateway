use std::{collections::HashMap, fs, io::Cursor, path::Path, sync::Mutex};

use mavlink::{
    common, peek_reader::PeekReader, read_v2_raw_message, MAVLinkV2MessageRaw,
    SigningConfig as MavlinkSigningConfig, SigningData,
};
use thiserror::Error;

use crate::mavlink_constants::MAVLINK2_SIGNED_INCOMPAT_FLAG;

pub const INSECURE_TEST_SIGNING_KEY: [u8; 32] = [
    0x10, 0x21, 0x32, 0x43, 0x54, 0x65, 0x76, 0x87, 0x98, 0xa9, 0xba, 0xcb, 0xdc, 0xed, 0xfe, 0x0f,
    0xf0, 0xe1, 0xd2, 0xc3, 0xb4, 0xa5, 0x96, 0x87, 0x78, 0x69, 0x5a, 0x4b, 0x3c, 0x2d, 0x1e, 0x0f,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SigningFrameInfo {
    pub system_id: u8,
    pub component_id: u8,
    pub link_id: u8,
    pub timestamp: u64,
    pub signature: [u8; 6],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SigningValidation {
    Unsigned,
    Valid(SigningFrameInfo),
    Invalid {
        info: Option<SigningFrameInfo>,
        reason: SigningRejectReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigningRejectReason {
    InvalidSignatureOrTimestamp,
    UnexpectedLinkId,
    Replay,
    ParseFailed,
}

impl SigningRejectReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidSignatureOrTimestamp => "invalid_signature_or_timestamp",
            Self::UnexpectedLinkId => "unexpected_link_id",
            Self::Replay => "replay",
            Self::ParseFailed => "parse_failed",
        }
    }
}

#[derive(Debug, Error)]
pub enum SigningError {
    #[error("could not read signing key file {path}: {source}")]
    ReadKey {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("signing key must be 32 bytes encoded as 64 hexadecimal characters")]
    InvalidKeyFormat,
}

pub struct SigningValidator {
    expected_link_id: u8,
    signing_data: SigningData,
    accepted_timestamps: Mutex<HashMap<(u8, u8, u8), u64>>,
}

impl SigningValidator {
    pub fn new(secret_key: [u8; 32], link_id: u8) -> Self {
        let config = MavlinkSigningConfig::new(secret_key, link_id, true, false);
        Self {
            expected_link_id: link_id,
            signing_data: SigningData::from_config(config),
            accepted_timestamps: Mutex::new(HashMap::new()),
        }
    }

    pub fn from_key_file(path: impl AsRef<Path>, link_id: u8) -> Result<Self, SigningError> {
        let key = load_hex_key(path)?;
        Ok(Self::new(key, link_id))
    }

    pub fn validate_datagram(&self, datagram: &[u8]) -> SigningValidation {
        let Ok(raw) = raw_v2(datagram) else {
            return SigningValidation::Invalid {
                info: None,
                reason: SigningRejectReason::ParseFailed,
            };
        };

        if !raw_is_signed(&raw) {
            return SigningValidation::Unsigned;
        }

        let info = signing_info(&raw);
        if info.link_id != self.expected_link_id {
            return SigningValidation::Invalid {
                info: Some(info),
                reason: SigningRejectReason::UnexpectedLinkId,
            };
        }

        if self.is_replay(&info) {
            return SigningValidation::Invalid {
                info: Some(info),
                reason: SigningRejectReason::Replay,
            };
        }

        if self.signing_data.verify_signature(&raw) {
            self.record_accepted(&info);
            SigningValidation::Valid(info)
        } else {
            SigningValidation::Invalid {
                info: Some(info),
                reason: SigningRejectReason::InvalidSignatureOrTimestamp,
            }
        }
    }

    fn is_replay(&self, info: &SigningFrameInfo) -> bool {
        let stream = (info.link_id, info.system_id, info.component_id);
        self.accepted_timestamps
            .lock()
            .expect("signing timestamp lock must not be poisoned")
            .get(&stream)
            .is_some_and(|last_timestamp| info.timestamp <= *last_timestamp)
    }

    fn record_accepted(&self, info: &SigningFrameInfo) {
        let stream = (info.link_id, info.system_id, info.component_id);
        self.accepted_timestamps
            .lock()
            .expect("signing timestamp lock must not be poisoned")
            .insert(stream, info.timestamp);
    }
}

pub fn raw_v2(datagram: &[u8]) -> Result<MAVLinkV2MessageRaw, mavlink::error::MessageReadError> {
    let mut reader = PeekReader::new(Cursor::new(datagram));
    read_v2_raw_message::<common::MavMessage, _>(&mut reader)
}

pub fn signing_info(raw: &MAVLinkV2MessageRaw) -> SigningFrameInfo {
    let mut signature = [0_u8; 6];
    signature.copy_from_slice(raw.signature_value());
    SigningFrameInfo {
        system_id: raw.system_id(),
        component_id: raw.component_id(),
        link_id: raw.signature_link_id(),
        timestamp: raw.signature_timestamp(),
        signature,
    }
}

pub fn raw_is_signed(raw: &MAVLinkV2MessageRaw) -> bool {
    raw.incompatibility_flags() & MAVLINK2_SIGNED_INCOMPAT_FLAG != 0
}

fn load_hex_key(path: impl AsRef<Path>) -> Result<[u8; 32], SigningError> {
    let path_ref = path.as_ref();
    let input = fs::read_to_string(path_ref).map_err(|source| SigningError::ReadKey {
        path: path_ref.display().to_string(),
        source,
    })?;
    parse_hex_key(&input)
}

fn parse_hex_key(input: &str) -> Result<[u8; 32], SigningError> {
    let normalized: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if normalized.len() != 64 || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(SigningError::InvalidKeyFormat);
    }

    let mut key = [0_u8; 32];
    for (index, chunk) in normalized.as_bytes().chunks_exact(2).enumerate() {
        let hex = std::str::from_utf8(chunk).map_err(|_| SigningError::InvalidKeyFormat)?;
        key[index] = u8::from_str_radix(hex, 16).map_err(|_| SigningError::InvalidKeyFormat)?;
    }
    Ok(key)
}

#[cfg(test)]
mod tests {
    use mavlink::{common, MavHeader};

    use super::*;

    const TEST_LINK_ID: u8 = 7;

    fn header() -> MavHeader {
        MavHeader {
            sequence: 9,
            system_id: 1,
            component_id: 1,
        }
    }

    fn heartbeat_message() -> common::MavMessage {
        common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode: 3,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        })
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

    fn unsigned_heartbeat() -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v2_msg(&mut bytes, header(), &heartbeat_message())
            .expect("fixture serializes");
        bytes
    }

    fn signed_heartbeat() -> Vec<u8> {
        sign_deterministically_with_link_id(&heartbeat_message(), TEST_LINK_ID)
    }

    fn signed_arm_command() -> Vec<u8> {
        sign_deterministically_with_link_id(&arm_command_message(), TEST_LINK_ID)
    }

    fn tampered_signature(mut packet: Vec<u8>) -> Vec<u8> {
        let last = packet.last_mut().expect("signed packet has signature byte");
        *last ^= 0x01;
        packet
    }

    fn signed_heartbeat_with_link_id(link_id: u8) -> Vec<u8> {
        sign_deterministically_with_link_id(&heartbeat_message(), link_id)
    }

    fn sign_deterministically_with_link_id(message: &common::MavMessage, link_id: u8) -> Vec<u8> {
        let mut raw = MAVLinkV2MessageRaw::new();
        raw.serialize_message_for_signing(header(), message);
        *raw.signature_link_id_mut() = link_id;
        raw.signature_timestamp_bytes_mut()
            .copy_from_slice(&[0xff; 6]);

        let mut signature = [0_u8; 6];
        raw.calculate_signature(&INSECURE_TEST_SIGNING_KEY, &mut signature);
        raw.signature_value_mut().copy_from_slice(&signature);
        raw.raw_bytes().to_vec()
    }

    #[test]
    fn validates_signed_heartbeat_with_test_key() {
        let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, TEST_LINK_ID);
        let validation = validator.validate_datagram(&signed_heartbeat());

        let SigningValidation::Valid(info) = validation else {
            panic!("expected valid signature, got {validation:?}");
        };
        assert_eq!(info.link_id, TEST_LINK_ID);
        assert_eq!(info.system_id, 1);
        assert_eq!(info.component_id, 1);
        assert!(info.timestamp > 0);
        assert_ne!(info.signature, [0_u8; 6]);
    }

    #[test]
    fn validates_signed_arm_command_fixture_with_test_key() {
        let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, TEST_LINK_ID);
        assert!(matches!(
            validator.validate_datagram(&signed_arm_command()),
            SigningValidation::Valid(_)
        ));
    }

    #[test]
    fn rejects_bad_signature_without_leaking_key_material() {
        let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, TEST_LINK_ID);
        let validation = validator.validate_datagram(&tampered_signature(signed_heartbeat()));

        assert!(matches!(
            validation,
            SigningValidation::Invalid {
                info: Some(_),
                reason: SigningRejectReason::InvalidSignatureOrTimestamp
            }
        ));
    }

    #[test]
    fn rejects_replayed_signed_heartbeat() {
        let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, TEST_LINK_ID);
        let packet = signed_heartbeat();

        assert!(matches!(
            validator.validate_datagram(&packet),
            SigningValidation::Valid(_)
        ));
        assert!(matches!(
            validator.validate_datagram(&packet),
            SigningValidation::Invalid {
                info: Some(_),
                reason: SigningRejectReason::Replay
            }
        ));
    }

    #[test]
    fn rejects_signed_packet_with_unexpected_link_id() {
        let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, TEST_LINK_ID);
        let validation = validator.validate_datagram(&signed_heartbeat_with_link_id(99));

        assert!(matches!(
            validation,
            SigningValidation::Invalid {
                info: Some(SigningFrameInfo { link_id: 99, .. }),
                reason: SigningRejectReason::UnexpectedLinkId
            }
        ));
    }

    #[test]
    fn classifies_unsigned_heartbeat_without_authentication_claim() {
        let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, TEST_LINK_ID);
        assert_eq!(
            validator.validate_datagram(&unsigned_heartbeat()),
            SigningValidation::Unsigned
        );
    }

    #[test]
    fn loads_hex_key_without_exposing_key_in_error() {
        let encoded = INSECURE_TEST_SIGNING_KEY
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();

        assert_eq!(
            parse_hex_key(&encoded).expect("valid hex key"),
            INSECURE_TEST_SIGNING_KEY
        );
        assert!(matches!(
            parse_hex_key("not-a-secret-key"),
            Err(SigningError::InvalidKeyFormat)
        ));
    }

    #[test]
    fn invalid_key_format_error_redacts_input_material() {
        let invalid_secret = "super-sensitive-lab-secret-that-must-not-appear";
        let err = parse_hex_key(invalid_secret).expect_err("invalid key format must fail");

        assert!(!err.to_string().contains(invalid_secret));
        assert_eq!(
            err.to_string(),
            "signing key must be 32 bytes encoded as 64 hexadecimal characters"
        );
    }
}
