#![no_main]

use libfuzzer_sys::fuzz_target;
use mavlink_rust_shield_gateway::{
    flight_state::FlightModeClassification,
    mavlink_codec::{decode_datagram, parse_error_decision, MavlinkSemantic},
    security_filter::{
        evaluate_command, Direction, MessageContext, PolicyDecision, SecurityPolicy, TransportKind,
    },
    signing::{SigningValidator, INSECURE_TEST_SIGNING_KEY},
};

fuzz_target!(|data: &[u8]| {
    let original = data.to_vec();
    let validator = SigningValidator::new(INSECURE_TEST_SIGNING_KEY, 7);
    let _ = validator.validate_datagram(data);
    assert_eq!(
        data, original.as_slice(),
        "signing validation must not mutate or reserialize the datagram"
    );

    let packet = match decode_datagram(data) {
        Ok(packet) => packet,
        Err(_) => {
            assert!(matches!(
                parse_error_decision(),
                PolicyDecision::DropInvalid(_)
            ));
            return;
        }
    };

    if packet.frame.signed {
        assert!(
            matches!(data.first(), Some(&mavlink::MAV_STX_V2)),
            "only MAVLink v2 frames can carry the signing incompatibility flag"
        );
    }

    match packet.semantic {
        MavlinkSemantic::Command(command) => {
            let policy = SecurityPolicy::new(Vec::new());
            let unknown_context = context(FlightModeClassification::Unknown);
            let unknown_decision = evaluate_command(&policy, &unknown_context, &command);

            if command.requires_known_flight_mode() {
                assert!(
                    !matches!(unknown_decision, PolicyDecision::Allow),
                    "critical or high-risk command must not be allowed when flight mode is unknown"
                );
            }
        }
        MavlinkSemantic::SetupSigning => {
            assert_eq!(
                packet.frame.message_id, 256,
                "SETUP_SIGNING must stay classified as sensitive semantic"
            );
        }
        MavlinkSemantic::Heartbeat(_) | MavlinkSemantic::Other { .. } => {}
    }
});

fn context(flight_mode: FlightModeClassification) -> MessageContext {
    MessageContext {
        direction: Direction::GcsToVehicle,
        transport: TransportKind::Udp,
        source_ip: None,
        flight_mode,
    }
}
