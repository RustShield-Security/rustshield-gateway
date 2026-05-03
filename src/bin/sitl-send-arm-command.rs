use std::{env, net::UdpSocket, time::Duration};

use mavlink::{common, MavHeader};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let target = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:14551".to_string());
    let bind = env::args()
        .nth(2)
        .unwrap_or_else(|| "127.0.0.1:0".to_string());

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
    )?;

    let socket = UdpSocket::bind(bind)?;
    socket.set_write_timeout(Some(Duration::from_secs(1)))?;
    socket.send_to(&bytes, &target)?;

    println!(
        "sent MAV_CMD_COMPONENT_ARM_DISARM arm attempt to {target}; source={}",
        socket.local_addr()?
    );

    Ok(())
}
