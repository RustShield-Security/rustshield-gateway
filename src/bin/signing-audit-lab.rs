use std::{
    fs,
    net::{SocketAddr, TcpListener},
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use mavlink::{common, MavHeader};
use mavlink_rust_shield_gateway::{
    config::{AppConfig, SigningPolicy},
    logging,
    signing::INSECURE_TEST_SIGNING_KEY,
    transport::UdpGateway,
};
use tokio::{net::UdpSocket, time::timeout};

const TEST_LINK_ID: u8 = 7;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logging::init("info")?;

    let lab_dir = lab_temp_dir()?;
    let key_path = lab_dir.join("signing-audit-lab.key");
    write_lab_key(&key_path)?;

    let gcs_receiver = UdpSocket::bind("127.0.0.1:0").await?;
    let vehicle_receiver = UdpSocket::bind("127.0.0.1:0").await?;

    let mut config = AppConfig::default();
    let (listen_gcs, listen_vehicle) = reserve_distinct_loopback_addrs()?;
    config.udp.listen_gcs = listen_gcs;
    config.udp.listen_vehicle = listen_vehicle;
    config.udp.gcs_addr = gcs_receiver.local_addr()?;
    config.udp.vehicle_addr = vehicle_receiver.local_addr()?;
    config.security.certified_ips = vec!["127.0.0.1".parse()?];
    config.signing.policy = SigningPolicy::Audit;
    config.signing.link_id = TEST_LINK_ID;
    config.signing.key_path = Some(key_path.clone());
    config.validate()?;

    let gateway = UdpGateway::bind(config).await?;
    let gcs_gateway_addr = gateway.gcs_listen_addr()?;
    let vehicle_gateway_addr = gateway.vehicle_listen_addr()?;

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let gateway_task = tokio::spawn(gateway.run_until_shutdown(async {
        let _ = shutdown_rx.await;
        Ok(())
    }));

    let vehicle_sender = UdpSocket::bind("127.0.0.1:0").await?;
    let valid_signed_heartbeat = signed_heartbeat([0xfd; 6]);
    send_and_expect_forward(
        "signed_valid_heartbeat",
        &vehicle_sender,
        vehicle_gateway_addr,
        &gcs_receiver,
        &valid_signed_heartbeat,
    )
    .await?;

    send_and_expect_forward(
        "signed_replay_heartbeat",
        &vehicle_sender,
        vehicle_gateway_addr,
        &gcs_receiver,
        &valid_signed_heartbeat,
    )
    .await?;

    let mut invalid_signed_heartbeat = signed_heartbeat([0xfe; 6]);
    tamper_signature(&mut invalid_signed_heartbeat);
    send_and_expect_forward(
        "signed_invalid_heartbeat",
        &vehicle_sender,
        vehicle_gateway_addr,
        &gcs_receiver,
        &invalid_signed_heartbeat,
    )
    .await?;

    let gcs_sender = UdpSocket::bind("127.0.0.1:0").await?;
    let unsigned_critical_command = unsigned_arm_command();
    send_and_expect_forward(
        "unsigned_critical_command",
        &gcs_sender,
        gcs_gateway_addr,
        &vehicle_receiver,
        &unsigned_critical_command,
    )
    .await?;

    tokio::time::sleep(Duration::from_millis(100)).await;
    let _ = shutdown_tx.send(());
    gateway_task.await??;

    fs::remove_file(&key_path)?;
    fs::remove_dir(&lab_dir)?;

    println!("signing_audit_lab.result=ok");
    println!("signing_audit_lab.forwarded_cases=4");
    println!("signing_audit_lab.enforce=false");
    println!("signing_audit_lab.temp_key_removed=true");

    Ok(())
}

async fn send_and_expect_forward(
    label: &str,
    sender: &UdpSocket,
    target: SocketAddr,
    receiver: &UdpSocket,
    packet: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    sender.send_to(packet, target).await?;

    let mut buffer = vec![0_u8; 2048];
    let (len, _) = timeout(Duration::from_secs(1), receiver.recv_from(&mut buffer)).await??;
    if &buffer[..len] != packet {
        return Err(format!("{label}: forwarded bytes differ from original packet").into());
    }

    println!("signing_audit_lab.forwarded={label}");
    Ok(())
}

fn lab_temp_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let path = std::env::temp_dir().join(format!(
        "mavlink-shield-signing-audit-lab-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir(&path)?;
    Ok(path)
}

fn write_lab_key(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let encoded_key = INSECURE_TEST_SIGNING_KEY
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    fs::write(path, encoded_key)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

fn reserve_distinct_loopback_addrs() -> Result<(SocketAddr, SocketAddr), Box<dyn std::error::Error>>
{
    let first = TcpListener::bind("127.0.0.1:0")?;
    let second = TcpListener::bind("127.0.0.1:0")?;
    Ok((first.local_addr()?, second.local_addr()?))
}

fn signed_heartbeat(timestamp_bytes: [u8; 6]) -> Vec<u8> {
    let message = common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
        custom_mode: 3,
        mavtype: common::MavType::MAV_TYPE_QUADROTOR,
        autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
        base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
        system_status: common::MavState::MAV_STATE_STANDBY,
        mavlink_version: 3,
    });

    let mut raw = mavlink::MAVLinkV2MessageRaw::new();
    raw.serialize_message_for_signing(
        MavHeader {
            sequence: 8,
            system_id: 1,
            component_id: 1,
        },
        &message,
    );
    *raw.signature_link_id_mut() = TEST_LINK_ID;
    raw.signature_timestamp_bytes_mut()
        .copy_from_slice(&timestamp_bytes);

    let mut signature = [0_u8; 6];
    raw.calculate_signature(&INSECURE_TEST_SIGNING_KEY, &mut signature);
    raw.signature_value_mut().copy_from_slice(&signature);
    raw.raw_bytes().to_vec()
}

fn tamper_signature(packet: &mut [u8]) {
    let last = packet
        .last_mut()
        .expect("signed MAVLink fixture has signature bytes");
    *last ^= 0x01;
}

fn unsigned_arm_command() -> Vec<u8> {
    let message = common::MavMessage::COMMAND_LONG(common::COMMAND_LONG_DATA {
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
    });

    let mut bytes = Vec::new();
    mavlink::write_v2_msg(
        &mut bytes,
        MavHeader {
            sequence: 42,
            system_id: 250,
            component_id: 190,
        },
        &message,
    )
    .expect("unsigned arm command fixture serializes");
    bytes
}
