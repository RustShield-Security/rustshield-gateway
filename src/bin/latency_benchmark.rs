use std::{
    hint::black_box,
    net::IpAddr,
    time::{Duration, Instant},
};

use mavlink::{common, MavHeader};
use mavlink_rust_shield_gateway::{
    flight_state::FlightModeClassification,
    mavlink_codec::{decode_datagram, MavlinkSemantic},
    security_filter::{evaluate_command, Direction, MessageContext, SecurityPolicy, TransportKind},
};

const ITERATIONS: usize = 20_000;

fn main() {
    let heartbeat = heartbeat_fixture(3);
    let command = arm_command_fixture();
    let invalid = b"not a mavlink frame".to_vec();
    let policy = SecurityPolicy::new(Vec::new());
    let source_ip: Option<IpAddr> = None;

    let parse_heartbeat = measure(ITERATIONS, || {
        black_box(decode_datagram(black_box(&heartbeat))).ok();
    });
    let parse_command_and_policy = measure(ITERATIONS, || {
        if let Ok(packet) = decode_datagram(black_box(&command)) {
            let MavlinkSemantic::Command(command) = packet.semantic else {
                return;
            };
            let context = MessageContext {
                direction: Direction::GcsToVehicle,
                transport: TransportKind::Udp,
                source_ip,
                flight_mode: FlightModeClassification::Unknown,
            };
            black_box(evaluate_command(&policy, &context, &command));
        }
    });
    let reject_invalid = measure(ITERATIONS, || {
        black_box(decode_datagram(black_box(&invalid))).err();
    });

    println!("latency benchmark: iterations={ITERATIONS}");
    print_report("parse_heartbeat", parse_heartbeat);
    print_report("parse_command_and_policy", parse_command_and_policy);
    print_report("reject_invalid", reject_invalid);
}

fn measure(iterations: usize, mut operation: impl FnMut()) -> Vec<Duration> {
    let mut samples = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let started = Instant::now();
        operation();
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    samples
}

fn print_report(name: &str, samples: Vec<Duration>) {
    let p50 = percentile(&samples, 50);
    let p95 = percentile(&samples, 95);
    let p99 = percentile(&samples, 99);
    let max = samples.last().copied().unwrap_or_default();
    let total: Duration = samples.iter().copied().sum();
    let throughput = samples.len() as f64 / total.as_secs_f64();

    println!(
        "{name}: p50={}ns/{:.3}us p95={}ns/{:.3}us p99={}ns/{:.3}us max={}ns/{:.3}us throughput={:.0} ops/s",
        p50.as_nanos(),
        nanos_to_us(p50),
        p95.as_nanos(),
        nanos_to_us(p95),
        p99.as_nanos(),
        nanos_to_us(p99),
        max.as_nanos(),
        nanos_to_us(max),
        throughput
    );
}

fn nanos_to_us(duration: Duration) -> f64 {
    duration.as_nanos() as f64 / 1_000.0
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    if samples.is_empty() {
        return Duration::default();
    }

    let index = ((samples.len() - 1) * percentile) / 100;
    samples[index]
}

fn header() -> MavHeader {
    MavHeader {
        sequence: 7,
        system_id: 1,
        component_id: 1,
    }
}

fn fixture(message: &common::MavMessage) -> Vec<u8> {
    let mut bytes = Vec::new();
    mavlink::write_v2_msg(&mut bytes, header(), message).expect("fixture serializes");
    bytes
}

fn heartbeat_fixture(custom_mode: u32) -> Vec<u8> {
    fixture(&common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
        custom_mode,
        mavtype: common::MavType::MAV_TYPE_QUADROTOR,
        autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
        base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
        system_status: common::MavState::MAV_STATE_STANDBY,
        mavlink_version: 3,
    }))
}

fn arm_command_fixture() -> Vec<u8> {
    fixture(&common::MavMessage::COMMAND_LONG(
        common::COMMAND_LONG_DATA {
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
        },
    ))
}
