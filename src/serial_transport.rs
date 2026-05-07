use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    os::fd::AsRawFd,
    path::Path,
    time::{Duration, Instant},
};

use thiserror::Error;

use crate::{config::SerialConfig, mavlink_codec::decode_datagram};

const MAVLINK_V1_STX: u8 = mavlink::MAV_STX;
const MAVLINK_V2_STX: u8 = mavlink::MAV_STX_V2;
const MAVLINK_V1_FIXED_LEN: usize = 8;
const MAVLINK_V2_FIXED_LEN: usize = 12;
const MAVLINK_V2_SIGNATURE_LEN: usize = 13;
const MAVLINK_V2_SIGNED_FLAG: u8 = 0x01;

#[derive(Debug, Error)]
pub enum SerialTransportError {
    #[error("could not open serial port {path}: {source}")]
    Open {
        path: String,
        #[source]
        source: io::Error,
    },

    #[error("serial I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("serial read timed out")]
    ReadTimeout,

    #[error("serial frame exceeds max_frame_size")]
    OversizedFrame,

    #[error("serial frame is not a valid MAVLink frame")]
    InvalidFrame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialForwardOutcome {
    Forwarded,
    DroppedInvalid,
}

pub struct SerialTransport {
    port: File,
    read_timeout: Duration,
    max_frame_size: usize,
}

impl SerialTransport {
    pub fn open(config: &SerialConfig) -> Result<Self, SerialTransportError> {
        let port = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&config.port)
            .map_err(|source| SerialTransportError::Open {
                path: config.port.display().to_string(),
                source,
            })?;
        Ok(Self::from_file(
            port,
            Duration::from_millis(config.read_timeout_ms),
            config.max_frame_size,
        ))
    }

    pub fn from_file(port: File, read_timeout: Duration, max_frame_size: usize) -> Self {
        Self {
            port,
            read_timeout,
            max_frame_size,
        }
    }

    pub fn reopen(&mut self, path: impl AsRef<Path>) -> Result<(), SerialTransportError> {
        self.port = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())
            .map_err(|source| SerialTransportError::Open {
                path: path.as_ref().display().to_string(),
                source,
            })?;
        Ok(())
    }

    pub fn read_frame(&mut self) -> Result<Vec<u8>, SerialTransportError> {
        let started = Instant::now();
        loop {
            let byte = self.read_byte_with_deadline(started)?;
            if matches!(byte, MAVLINK_V1_STX | MAVLINK_V2_STX) {
                return self.read_frame_after_marker(byte, started);
            }
        }
    }

    pub fn write_frame(&mut self, frame: &[u8]) -> Result<(), SerialTransportError> {
        self.port.write_all(frame)?;
        self.port.flush()?;
        Ok(())
    }

    pub fn forward_one_frame_to<W: Write>(
        &mut self,
        destination: &mut W,
    ) -> Result<SerialForwardOutcome, SerialTransportError> {
        let frame = self.read_frame()?;
        if decode_datagram(&frame).is_err() {
            return Ok(SerialForwardOutcome::DroppedInvalid);
        }
        destination.write_all(&frame)?;
        Ok(SerialForwardOutcome::Forwarded)
    }

    fn read_frame_after_marker(
        &mut self,
        marker: u8,
        started: Instant,
    ) -> Result<Vec<u8>, SerialTransportError> {
        let payload_len = self.read_byte_with_deadline(started)? as usize;
        let mut frame = vec![marker, payload_len as u8];

        let remaining = match marker {
            MAVLINK_V1_STX => MAVLINK_V1_FIXED_LEN - 2 + payload_len,
            MAVLINK_V2_STX => {
                let incompat_flags = self.read_byte_with_deadline(started)?;
                frame.push(incompat_flags);
                let signature_len = if incompat_flags & MAVLINK_V2_SIGNED_FLAG != 0 {
                    MAVLINK_V2_SIGNATURE_LEN
                } else {
                    0
                };
                MAVLINK_V2_FIXED_LEN - 3 + payload_len + signature_len
            }
            _ => unreachable!("marker checked by caller"),
        };

        let total_len = frame.len() + remaining;
        if total_len > self.max_frame_size {
            return Err(SerialTransportError::OversizedFrame);
        }

        for _ in 0..remaining {
            frame.push(self.read_byte_with_deadline(started)?);
        }

        Ok(frame)
    }

    fn read_byte_with_deadline(&mut self, started: Instant) -> Result<u8, SerialTransportError> {
        let elapsed = started.elapsed();
        let Some(remaining) = self.read_timeout.checked_sub(elapsed) else {
            return Err(SerialTransportError::ReadTimeout);
        };

        wait_readable(self.port.as_raw_fd(), remaining)?;

        let mut byte = [0_u8; 1];
        match self.port.read_exact(&mut byte) {
            Ok(()) => Ok(byte[0]),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                Err(SerialTransportError::ReadTimeout)
            }
            Err(error) => Err(SerialTransportError::Io(error)),
        }
    }
}

fn wait_readable(fd: i32, timeout: Duration) -> Result<(), SerialTransportError> {
    #[repr(C)]
    struct PollFd {
        fd: i32,
        events: i16,
        revents: i16,
    }

    const POLLIN: i16 = 0x0001;

    unsafe extern "C" {
        fn poll(fds: *mut PollFd, nfds: usize, timeout: i32) -> i32;
    }

    let timeout_ms = timeout.as_millis().min(i32::MAX as u128) as i32;
    let mut poll_fd = PollFd {
        fd,
        events: POLLIN,
        revents: 0,
    };

    let result = unsafe { poll(&mut poll_fd, 1, timeout_ms) };
    if result == 0 {
        return Err(SerialTransportError::ReadTimeout);
    }
    if result < 0 {
        return Err(SerialTransportError::Io(io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{
        ffi::CStr,
        io::Write,
        os::{
            fd::{FromRawFd, RawFd},
            raw::{c_char, c_int, c_void},
        },
        thread,
    };

    use mavlink::{common, MavHeader};

    use super::*;

    unsafe extern "C" {
        fn openpty(
            amaster: *mut c_int,
            aslave: *mut c_int,
            name: *mut c_char,
            termp: *const c_void,
            winp: *const c_void,
        ) -> c_int;
    }

    struct PtyPair {
        master: File,
        slave: File,
        slave_path: String,
    }

    fn open_virtual_pty() -> PtyPair {
        let mut master: RawFd = -1;
        let mut slave: RawFd = -1;
        let mut name = [0_i8; 128];
        let result = unsafe {
            openpty(
                &mut master,
                &mut slave,
                name.as_mut_ptr(),
                std::ptr::null(),
                std::ptr::null(),
            )
        };
        assert_eq!(result, 0, "openpty failed: {}", io::Error::last_os_error());
        let slave_path = unsafe { CStr::from_ptr(name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        std::process::Command::new("stty")
            .args(["-F", &slave_path, "raw", "-echo"])
            .status()
            .expect("stty is available")
            .success()
            .then_some(())
            .expect("PTY can be configured raw");
        PtyPair {
            master: unsafe { File::from_raw_fd(master) },
            slave: unsafe { File::from_raw_fd(slave) },
            slave_path,
        }
    }

    fn fixture(message: &common::MavMessage) -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v2_msg(
            &mut bytes,
            MavHeader {
                sequence: 7,
                system_id: 1,
                component_id: 1,
            },
            message,
        )
        .expect("fixture serializes");
        bytes
    }

    fn v1_fixture(message: &common::MavMessage) -> Vec<u8> {
        let mut bytes = Vec::new();
        mavlink::write_v1_msg(
            &mut bytes,
            MavHeader {
                sequence: 8,
                system_id: 1,
                component_id: 1,
            },
            message,
        )
        .expect("fixture serializes");
        bytes
    }

    fn heartbeat() -> common::MavMessage {
        common::MavMessage::HEARTBEAT(common::HEARTBEAT_DATA {
            custom_mode: 0,
            mavtype: common::MavType::MAV_TYPE_QUADROTOR,
            autopilot: common::MavAutopilot::MAV_AUTOPILOT_ARDUPILOTMEGA,
            base_mode: common::MavModeFlag::MAV_MODE_FLAG_CUSTOM_MODE_ENABLED,
            system_status: common::MavState::MAV_STATE_STANDBY,
            mavlink_version: 3,
        })
    }

    fn signed_fixture() -> Vec<u8> {
        let mut raw = mavlink::MAVLinkV2MessageRaw::new();
        raw.serialize_message_for_signing(
            MavHeader {
                sequence: 9,
                system_id: 1,
                component_id: 1,
            },
            &heartbeat(),
        );
        *raw.signature_link_id_mut() = 7;
        raw.signature_timestamp_bytes_mut()
            .copy_from_slice(&[0xff; 6]);
        let mut signature = [0_u8; 6];
        raw.calculate_signature(&crate::signing::INSECURE_TEST_SIGNING_KEY, &mut signature);
        raw.signature_value_mut().copy_from_slice(&signature);
        raw.raw_bytes().to_vec()
    }

    #[test]
    fn serial_forwards_valid_mavlink_frame_from_virtual_pty() {
        let pty = open_virtual_pty();
        let frame = fixture(&heartbeat());
        let mut master = pty.master.try_clone().expect("clone master");
        let mut transport = SerialTransport::from_file(pty.slave, Duration::from_millis(500), 280);

        thread::spawn(move || master.write_all(&frame).expect("write frame"))
            .join()
            .unwrap();

        let mut output = Vec::new();
        let outcome = transport
            .forward_one_frame_to(&mut output)
            .expect("forward succeeds");

        assert_eq!(outcome, SerialForwardOutcome::Forwarded);
        assert_eq!(output, fixture(&heartbeat()));
    }

    #[test]
    fn serial_drops_invalid_frame_from_virtual_pty() {
        let pty = open_virtual_pty();
        let mut invalid = fixture(&heartbeat());
        let last = invalid.last_mut().expect("frame has checksum");
        *last ^= 0x01;
        let mut master = pty.master.try_clone().expect("clone master");
        let mut transport = SerialTransport::from_file(pty.slave, Duration::from_millis(500), 280);

        thread::spawn(move || master.write_all(&invalid).expect("write invalid"))
            .join()
            .unwrap();

        let mut output = Vec::new();
        let outcome = transport
            .forward_one_frame_to(&mut output)
            .expect("invalid frame is dropped without I/O failure");

        assert_eq!(outcome, SerialForwardOutcome::DroppedInvalid);
        assert!(output.is_empty());
    }

    #[test]
    fn serial_preserves_mavlink_v1_and_v2_bytes() {
        for frame in [v1_fixture(&heartbeat()), fixture(&heartbeat())] {
            let pty = open_virtual_pty();
            let mut master = pty.master.try_clone().expect("clone master");
            let mut transport =
                SerialTransport::from_file(pty.slave, Duration::from_millis(500), 280);
            let expected = frame.clone();

            thread::spawn(move || master.write_all(&frame).expect("write frame"))
                .join()
                .unwrap();

            assert_eq!(transport.read_frame().expect("frame reads"), expected);
        }
    }

    #[test]
    fn serial_handles_signed_frame_transparently() {
        let pty = open_virtual_pty();
        let frame = signed_fixture();
        let mut master = pty.master.try_clone().expect("clone master");
        let mut transport = SerialTransport::from_file(pty.slave, Duration::from_millis(500), 280);
        let expected = frame.clone();

        thread::spawn(move || master.write_all(&frame).expect("write signed frame"))
            .join()
            .unwrap();

        let read = transport.read_frame().expect("signed frame reads");
        assert_eq!(read, expected);
        assert!(decode_datagram(&read).expect("signed parses").frame.signed);
    }

    #[test]
    fn serial_reconnects_to_virtual_pty_path() {
        let first = open_virtual_pty();
        let second = open_virtual_pty();
        let mut first_master = first.master.try_clone().expect("clone master");
        let mut second_master = second.master.try_clone().expect("clone master");
        let frame1 = fixture(&heartbeat());
        let frame2 = v1_fixture(&heartbeat());
        let expected1 = frame1.clone();
        let expected2 = frame2.clone();
        let mut transport =
            SerialTransport::from_file(first.slave, Duration::from_millis(500), 280);

        first_master.write_all(&frame1).expect("write first");
        assert_eq!(transport.read_frame().expect("first reads"), expected1);

        transport
            .reopen(&second.slave_path)
            .expect("reopen second pty");
        second_master.write_all(&frame2).expect("write second");
        assert_eq!(transport.read_frame().expect("second reads"), expected2);
    }

    #[test]
    fn serial_read_timeout_is_reported() {
        let pty = open_virtual_pty();
        let mut transport = SerialTransport::from_file(pty.slave, Duration::from_millis(10), 280);

        let err = transport.read_frame().expect_err("empty pty times out");

        assert!(matches!(err, SerialTransportError::ReadTimeout));
    }

    #[test]
    fn serial_oversized_frame_is_rejected() {
        let pty = open_virtual_pty();
        let frame = fixture(&heartbeat());
        let mut master = pty.master.try_clone().expect("clone master");
        let mut transport = SerialTransport::from_file(pty.slave, Duration::from_millis(500), 8);

        thread::spawn(move || master.write_all(&frame).expect("write frame"))
            .join()
            .unwrap();

        let err = transport
            .read_frame()
            .expect_err("frame exceeds configured size");

        assert!(matches!(err, SerialTransportError::OversizedFrame));
    }

    #[test]
    fn serial_handles_half_frame_across_reads_and_multiple_frames() {
        let pty = open_virtual_pty();
        let frame1 = fixture(&heartbeat());
        let frame2 = v1_fixture(&heartbeat());
        let expected1 = frame1.clone();
        let expected2 = frame2.clone();
        let split_at = 3;
        let mut master = pty.master.try_clone().expect("clone master");
        let mut transport = SerialTransport::from_file(pty.slave, Duration::from_millis(500), 280);

        master
            .write_all(&frame1[..split_at])
            .expect("write first half");
        master
            .write_all(&frame1[split_at..])
            .expect("write second half");
        master.write_all(&frame2).expect("write second frame");

        assert_eq!(transport.read_frame().expect("first reads"), expected1);
        assert_eq!(transport.read_frame().expect("second reads"), expected2);
    }

    #[test]
    fn serial_write_frame_reports_write_failure() {
        let sink = OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .expect("/dev/full is available on Linux");
        let mut transport = SerialTransport::from_file(sink, Duration::from_millis(500), 280);

        let err = transport
            .write_frame(&fixture(&heartbeat()))
            .expect_err("/dev/full causes write failure");

        assert!(matches!(err, SerialTransportError::Io(_)));
    }
}
