use std::{
    fs,
    net::{SocketAddr, UdpSocket},
    path::PathBuf,
    time::Duration,
};

use clap::{Parser, ValueEnum};
use mavlink::{common, MavHeader};

const DEFAULT_TIMESTAMP_BYTES: [u8; 6] = [0xff; 6];

#[derive(Debug, Parser)]
#[command(
    about = "Send controlled MAVLink signing enforce SITL cases to the gateway",
    version
)]
struct Cli {
    #[arg(long, default_value = "127.0.0.1:14551")]
    target: SocketAddr,

    #[arg(long, default_value = "127.0.0.1:0")]
    bind: SocketAddr,

    #[arg(long, value_enum)]
    case: EnforceCase,

    #[arg(long)]
    key_path: Option<PathBuf>,

    #[arg(long, default_value_t = 7)]
    link_id: u8,

    #[arg(long, default_value_t = 1)]
    repeat: u8,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum EnforceCase {
    UnsignedArm,
    SignedArmValid,
    SignedArmInvalid,
    SignedArmReplay,
    SignedArmUnexpectedLink,
    SetupSigning,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let repeat = if matches!(cli.case, EnforceCase::SignedArmReplay) {
        cli.repeat.max(2)
    } else {
        cli.repeat
    };
    let packet = packet_for_case(&cli)?;

    let socket = UdpSocket::bind(cli.bind)?;
    socket.set_write_timeout(Some(Duration::from_secs(1)))?;

    for attempt in 1..=repeat {
        socket.send_to(&packet, cli.target)?;
        println!(
            "sitl_enforce_signing_case.sent case={} attempt={} target={} source={} bytes={}",
            cli.case.as_str(),
            attempt,
            cli.target,
            socket.local_addr()?,
            packet.len()
        );
    }

    Ok(())
}

fn packet_for_case(cli: &Cli) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    match cli.case {
        EnforceCase::UnsignedArm => Ok(unsigned_arm_command()),
        EnforceCase::SignedArmValid | EnforceCase::SignedArmReplay => {
            let key = load_hex_key(required_key_path(cli)?)?;
            Ok(signed_arm_command(&key, cli.link_id))
        }
        EnforceCase::SignedArmInvalid => {
            let key = load_hex_key(required_key_path(cli)?)?;
            let mut packet = signed_arm_command(&key, cli.link_id);
            tamper_signature(&mut packet);
            Ok(packet)
        }
        EnforceCase::SignedArmUnexpectedLink => {
            let key = load_hex_key(required_key_path(cli)?)?;
            Ok(signed_arm_command(&key, cli.link_id.wrapping_add(1)))
        }
        EnforceCase::SetupSigning => Ok(setup_signing()),
    }
}

fn required_key_path(cli: &Cli) -> Result<&PathBuf, Box<dyn std::error::Error>> {
    cli.key_path
        .as_ref()
        .ok_or_else(|| "--key-path is required for signed cases".into())
}

fn unsigned_arm_command() -> Vec<u8> {
    let mut bytes = Vec::new();
    mavlink::write_v2_msg(&mut bytes, gcs_header(), &arm_command_message())
        .expect("unsigned arm command fixture serializes");
    bytes
}

fn signed_arm_command(secret_key: &[u8; 32], link_id: u8) -> Vec<u8> {
    let mut raw = mavlink::MAVLinkV2MessageRaw::new();
    raw.serialize_message_for_signing(gcs_header(), &arm_command_message());
    *raw.signature_link_id_mut() = link_id;
    raw.signature_timestamp_bytes_mut()
        .copy_from_slice(&DEFAULT_TIMESTAMP_BYTES);

    let mut signature = [0_u8; 6];
    raw.calculate_signature(secret_key, &mut signature);
    raw.signature_value_mut().copy_from_slice(&signature);
    raw.raw_bytes().to_vec()
}

fn setup_signing() -> Vec<u8> {
    let mut bytes = Vec::new();
    mavlink::write_v2_msg(
        &mut bytes,
        gcs_header(),
        &common::MavMessage::SETUP_SIGNING(common::SETUP_SIGNING_DATA::default()),
    )
    .expect("setup signing fixture serializes");
    bytes
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

fn gcs_header() -> MavHeader {
    MavHeader {
        sequence: 42,
        system_id: 250,
        component_id: 190,
    }
}

fn tamper_signature(packet: &mut [u8]) {
    let last = packet
        .last_mut()
        .expect("signed MAVLink fixture has signature bytes");
    *last ^= 0x01;
}

fn load_hex_key(path: &PathBuf) -> Result<[u8; 32], Box<dyn std::error::Error>> {
    let input = fs::read_to_string(path)?;
    let normalized: String = input.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    if normalized.len() != 64 || !normalized.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("signing key must be 32 bytes encoded as 64 hexadecimal characters".into());
    }

    let mut key = [0_u8; 32];
    for (index, chunk) in normalized.as_bytes().chunks_exact(2).enumerate() {
        let hex = std::str::from_utf8(chunk)?;
        key[index] = u8::from_str_radix(hex, 16)?;
    }
    Ok(key)
}

impl EnforceCase {
    fn as_str(self) -> &'static str {
        match self {
            Self::UnsignedArm => "unsigned-arm",
            Self::SignedArmValid => "signed-arm-valid",
            Self::SignedArmInvalid => "signed-arm-invalid",
            Self::SignedArmReplay => "signed-arm-replay",
            Self::SignedArmUnexpectedLink => "signed-arm-unexpected-link",
            Self::SetupSigning => "setup-signing",
        }
    }
}
