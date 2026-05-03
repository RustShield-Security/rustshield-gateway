use std::{fs, path::Path};

use mavlink::{common, MavHeader};

use mavlink_rust_shield_gateway::signing::INSECURE_TEST_SIGNING_KEY;

fn main() -> std::io::Result<()> {
    let corpus_dir = Path::new("fuzz/corpus/fuzz_mavlink_frame");
    fs::create_dir_all(corpus_dir)?;

    for (name, bytes) in fixtures() {
        fs::write(corpus_dir.join(name), bytes)?;
    }

    println!(
        "exported deterministic fuzz corpus to {}",
        corpus_dir.display()
    );
    Ok(())
}

#[allow(deprecated)]
fn fixtures() -> Vec<(&'static str, Vec<u8>)> {
    let heartbeat_v1 = v1(&heartbeat(3), header(9, 1, 1));
    let heartbeat_v2 = v2(&heartbeat(5), header(10, 42, 191));
    let signed_heartbeat_v2 = signed_v2(&heartbeat(3), header(11, 7, 1));
    let signed_heartbeat_bad_signature_v2 = tamper_last_byte(signed_heartbeat_v2.clone());
    let signed_heartbeat_unexpected_link_id_v2 =
        signed_v2_with_link_id(&heartbeat(3), header(12, 7, 1), 8);
    let signed_heartbeat_replay_like_v2 = signed_v2(&heartbeat(3), header(13, 7, 1));
    let arm_command_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM, 1.0, 0.0),
        header(14, 255, 190),
    );
    let force_arm_command_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_COMPONENT_ARM_DISARM, 1.0, 21196.0),
        header(15, 255, 190),
    );
    let takeoff_command_long_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_NAV_TAKEOFF, 0.0, 0.0),
        header(16, 255, 190),
    );
    let land_command_long_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_NAV_LAND, 0.0, 0.0),
        header(17, 255, 190),
    );
    let set_mode_command_long_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_DO_SET_MODE, 0.0, 0.0),
        header(18, 255, 190),
    );
    let mission_start_command_long_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_MISSION_START, 0.0, 0.0),
        header(19, 255, 190),
    );
    let reboot_command_long_v2 = v2(
        &command_long(common::MavCmd::MAV_CMD_PREFLIGHT_REBOOT_SHUTDOWN, 0.0, 0.0),
        header(20, 255, 190),
    );
    let takeoff_command_int_v2 = v2(
        &command_int(common::MavCmd::MAV_CMD_NAV_TAKEOFF),
        header(21, 255, 190),
    );
    let reposition_v2 = v2(
        &command_int(common::MavCmd::MAV_CMD_DO_REPOSITION),
        header(22, 255, 190),
    );
    let set_mode_v2 = v2(
        &common::MavMessage::SET_MODE(common::SET_MODE_DATA::default()),
        header(23, 255, 190),
    );
    let mission_count_v2 = v2(
        &common::MavMessage::MISSION_COUNT(common::MISSION_COUNT_DATA {
            target_system: 1,
            target_component: 1,
            count: 2,
        }),
        header(24, 255, 190),
    );
    let mission_item_v2 = v2(
        &common::MavMessage::MISSION_ITEM(common::MISSION_ITEM_DATA::default()),
        header(25, 255, 190),
    );
    let mission_item_int_v2 = v2(
        &common::MavMessage::MISSION_ITEM_INT(common::MISSION_ITEM_INT_DATA::default()),
        header(26, 255, 190),
    );
    let mission_clear_all_v2 = v2(
        &common::MavMessage::MISSION_CLEAR_ALL(common::MISSION_CLEAR_ALL_DATA::default()),
        header(27, 255, 190),
    );
    let mission_set_current_v2 = v2(
        &common::MavMessage::MISSION_SET_CURRENT(common::MISSION_SET_CURRENT_DATA::default()),
        header(28, 255, 190),
    );
    let mission_write_partial_list_v2 = v2(
        &common::MavMessage::MISSION_WRITE_PARTIAL_LIST(
            common::MISSION_WRITE_PARTIAL_LIST_DATA::default(),
        ),
        header(29, 255, 190),
    );
    let param_set_v2 = v2(
        &common::MavMessage::PARAM_SET(common::PARAM_SET_DATA {
            target_system: 1,
            target_component: 1,
            param_id: "ARMING_CHECK".into(),
            param_value: 0.0,
            param_type: common::MavParamType::MAV_PARAM_TYPE_REAL32,
        }),
        header(30, 255, 190),
    );
    let manual_control_v2 = v2(
        &common::MavMessage::MANUAL_CONTROL(common::MANUAL_CONTROL_DATA {
            target: 1,
            x: 100,
            y: -100,
            z: 500,
            r: 0,
            buttons: 0,
        }),
        header(31, 255, 190),
    );
    let rc_channels_override_v2 = v2(
        &common::MavMessage::RC_CHANNELS_OVERRIDE(common::RC_CHANNELS_OVERRIDE_DATA::default()),
        header(32, 255, 190),
    );
    let setup_signing_v2 = v2(
        &common::MavMessage::SETUP_SIGNING(common::SETUP_SIGNING_DATA::default()),
        header(33, 255, 190),
    );
    let mut truncated = heartbeat_v2.clone();
    truncated.truncate(truncated.len().saturating_sub(2));
    let mut trailing = heartbeat_v2.clone();
    trailing.extend_from_slice(&[0xaa, 0x55]);
    let mut bad_crc = heartbeat_v2.clone();
    if let Some(last) = bad_crc.last_mut() {
        *last ^= 0x01;
    }
    let mut unknown_message_id = heartbeat_v2.clone();
    unknown_message_id[7] = 0xfe;
    unknown_message_id[8] = 0xff;
    unknown_message_id[9] = 0x00;
    let payload_limit_probe = vec![
        mavlink::MAV_STX_V2,
        255,
        0,
        0,
        34,
        255,
        190,
        0xff,
        0xff,
        0x00,
    ];

    vec![
        ("heartbeat-v1.bin", heartbeat_v1),
        ("heartbeat-v2.bin", heartbeat_v2),
        ("heartbeat-v2-signed.bin", signed_heartbeat_v2),
        (
            "heartbeat-v2-signed-bad-signature.bin",
            signed_heartbeat_bad_signature_v2,
        ),
        (
            "heartbeat-v2-signed-unexpected-link-id.bin",
            signed_heartbeat_unexpected_link_id_v2,
        ),
        (
            "heartbeat-v2-signed-replay-like.bin",
            signed_heartbeat_replay_like_v2,
        ),
        ("command-long-arm-v2.bin", arm_command_v2),
        ("command-long-force-arm-v2.bin", force_arm_command_v2),
        ("command-long-takeoff-v2.bin", takeoff_command_long_v2),
        ("command-long-land-v2.bin", land_command_long_v2),
        ("command-long-do-set-mode-v2.bin", set_mode_command_long_v2),
        (
            "command-long-mission-start-v2.bin",
            mission_start_command_long_v2,
        ),
        ("command-long-reboot-v2.bin", reboot_command_long_v2),
        ("command-int-takeoff-v2.bin", takeoff_command_int_v2),
        ("command-int-reposition-v2.bin", reposition_v2),
        ("set-mode-v2.bin", set_mode_v2),
        ("mission-count-v2.bin", mission_count_v2),
        ("mission-item-v2.bin", mission_item_v2),
        ("mission-item-int-v2.bin", mission_item_int_v2),
        ("mission-clear-all-v2.bin", mission_clear_all_v2),
        ("mission-set-current-v2.bin", mission_set_current_v2),
        (
            "mission-write-partial-list-v2.bin",
            mission_write_partial_list_v2,
        ),
        ("param-set-v2.bin", param_set_v2),
        ("manual-control-v2.bin", manual_control_v2),
        ("rc-channels-override-v2.bin", rc_channels_override_v2),
        ("setup-signing-v2.bin", setup_signing_v2),
        ("truncated-heartbeat-v2.bin", truncated),
        ("trailing-heartbeat-v2.bin", trailing),
        ("bad-crc-heartbeat-v2.bin", bad_crc),
        ("unknown-message-id-v2.bin", unknown_message_id),
        ("payload-limit-probe-v2.bin", payload_limit_probe),
    ]
}

fn header(sequence: u8, system_id: u8, component_id: u8) -> MavHeader {
    MavHeader {
        sequence,
        system_id,
        component_id,
    }
}

fn v1(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
    let mut bytes = Vec::new();
    mavlink::write_v1_msg(&mut bytes, header, message).expect("fixture serializes as MAVLink v1");
    bytes
}

fn v2(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
    let mut bytes = Vec::new();
    mavlink::write_v2_msg(&mut bytes, header, message).expect("fixture serializes as MAVLink v2");
    bytes
}

fn signed_v2(message: &common::MavMessage, header: MavHeader) -> Vec<u8> {
    signed_v2_with_link_id(message, header, 7)
}

fn signed_v2_with_link_id(message: &common::MavMessage, header: MavHeader, link_id: u8) -> Vec<u8> {
    let mut raw = mavlink::MAVLinkV2MessageRaw::new();
    raw.serialize_message_for_signing(header, message);
    *raw.signature_link_id_mut() = link_id;
    raw.signature_timestamp_bytes_mut()
        .copy_from_slice(&[0x10, 0x20, 0x30, 0x40, 0x50, 0x60]);
    let mut signature = [0_u8; 6];
    raw.calculate_signature(&INSECURE_TEST_SIGNING_KEY, &mut signature);
    raw.signature_value_mut().copy_from_slice(&signature);
    raw.raw_bytes().to_vec()
}

fn tamper_last_byte(mut bytes: Vec<u8>) -> Vec<u8> {
    if let Some(last) = bytes.last_mut() {
        *last ^= 0x01;
    }
    bytes
}

fn heartbeat(custom_mode: u32) -> common::MavMessage {
    common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
        custom_mode,
        mavtype: common::MavType::MAV_TYPE_QUADROTOR,
        autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
        base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
        system_status: common::MavState::MAV_STATE_STANDBY,
        mavlink_version: 3,
    })
}

fn command_long(command: common::MavCmd, param1: f32, param2: f32) -> common::MavMessage {
    common::MavMessage::COMMAND_LONG(common::COMMAND_LONG_DATA {
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
    })
}

fn command_int(command: common::MavCmd) -> common::MavMessage {
    common::MavMessage::COMMAND_INT(common::COMMAND_INT_DATA {
        param1: 0.0,
        param2: 0.0,
        param3: 0.0,
        param4: 0.0,
        x: 407397000,
        y: -73990000,
        z: 40.0,
        command,
        target_system: 1,
        target_component: 1,
        frame: common::MavFrame::MAV_FRAME_GLOBAL_RELATIVE_ALT,
        current: 0,
        autocontinue: 0,
    })
}
