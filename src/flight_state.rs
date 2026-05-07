#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Autopilot {
    ArduPilot,
    Px4,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VehicleFamily {
    ArduCopter,
    ArduPlane,
    Rover,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlightModeClassification {
    Automatic,
    NotAutomatic,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeObservation {
    pub autopilot: Autopilot,
    pub vehicle_family: VehicleFamily,
    pub custom_mode: Option<u32>,
}

impl ModeObservation {
    pub fn classify(self) -> FlightModeClassification {
        classify_mode(self.autopilot, self.vehicle_family, self.custom_mode)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeartbeatObservation {
    pub autopilot: Autopilot,
    pub vehicle_family: VehicleFamily,
    pub base_mode: u8,
    pub custom_mode: Option<u32>,
    pub source_system: u8,
    pub source_component: u8,
}

impl HeartbeatObservation {
    pub fn mode_observation(self) -> ModeObservation {
        ModeObservation {
            autopilot: self.autopilot,
            vehicle_family: self.vehicle_family,
            custom_mode: self.custom_mode,
        }
    }

    pub fn classify(self) -> FlightModeClassification {
        self.mode_observation().classify()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlightModeSnapshot {
    pub observation: HeartbeatObservation,
    pub classification: FlightModeClassification,
    pub mode_name: Option<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeChangedEvent {
    pub audit_event: &'static str,
    pub autopilot: Autopilot,
    pub vehicle_family: VehicleFamily,
    pub base_mode: u8,
    pub custom_mode: Option<u32>,
    pub mode_name: Option<&'static str>,
    pub classification: FlightModeClassification,
    pub previous_classification: Option<FlightModeClassification>,
    pub source_system: u8,
    pub source_component: u8,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FlightState {
    current_mode: Option<FlightModeSnapshot>,
}

impl FlightState {
    pub fn current_mode(&self) -> Option<FlightModeSnapshot> {
        self.current_mode
    }

    pub fn update_heartbeat(
        &mut self,
        observation: HeartbeatObservation,
    ) -> Option<ModeChangedEvent> {
        let classification = observation.classify();
        let mode_name = match (
            observation.autopilot,
            observation.vehicle_family,
            observation.custom_mode,
        ) {
            (Autopilot::ArduPilot, VehicleFamily::ArduCopter, Some(mode)) => {
                arducopter_mode_name(mode)
            }
            _ => None,
        };
        let previous = self.current_mode;
        let snapshot = FlightModeSnapshot {
            observation,
            classification,
            mode_name,
        };

        self.current_mode = Some(snapshot);

        if previous.is_some_and(|previous| previous == snapshot) {
            return None;
        }

        Some(ModeChangedEvent {
            audit_event: "flight_state.mode_changed",
            autopilot: observation.autopilot,
            vehicle_family: observation.vehicle_family,
            base_mode: observation.base_mode,
            custom_mode: observation.custom_mode,
            mode_name,
            classification,
            previous_classification: previous.map(|snapshot| snapshot.classification),
            source_system: observation.source_system,
            source_component: observation.source_component,
        })
    }
}

pub fn classify_mode(
    autopilot: Autopilot,
    vehicle_family: VehicleFamily,
    custom_mode: Option<u32>,
) -> FlightModeClassification {
    if autopilot != Autopilot::ArduPilot || vehicle_family != VehicleFamily::ArduCopter {
        return FlightModeClassification::Unknown;
    }

    match custom_mode {
        Some(3) => FlightModeClassification::Automatic,
        Some(mode) if is_known_arducopter_mode(mode) => FlightModeClassification::NotAutomatic,
        _ => FlightModeClassification::Unknown,
    }
}

pub fn arducopter_mode_name(custom_mode: u32) -> Option<&'static str> {
    Some(match custom_mode {
        0 => "Stabilize",
        1 => "Acro",
        2 => "AltHold",
        3 => "Auto",
        4 => "Guided",
        5 => "Loiter",
        6 => "RTL",
        7 => "Circle",
        9 => "Land",
        11 => "Drift",
        13 => "Sport",
        14 => "Flip",
        15 => "AutoTune",
        16 => "PosHold",
        17 => "Brake",
        18 => "Throw",
        19 => "Avoid_ADSB",
        20 => "Guided_NoGPS",
        21 => "Smart_RTL",
        22 => "FlowHold",
        23 => "Follow",
        24 => "ZigZag",
        25 => "SystemID",
        26 => "Heli_Autorotate",
        27 => "Auto RTL",
        28 => "Turtle",
        _ => return None,
    })
}

pub fn is_known_arducopter_mode(custom_mode: u32) -> bool {
    arducopter_mode_name(custom_mode).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mode_ap_u_001_arducopter_auto_is_automatic() {
        let classification =
            classify_mode(Autopilot::ArduPilot, VehicleFamily::ArduCopter, Some(3));

        assert_eq!(classification, FlightModeClassification::Automatic);
    }

    #[test]
    fn mode_ap_u_002_arducopter_stabilize_is_not_automatic() {
        let classification =
            classify_mode(Autopilot::ArduPilot, VehicleFamily::ArduCopter, Some(0));

        assert_eq!(classification, FlightModeClassification::NotAutomatic);
    }

    #[test]
    fn mode_ap_u_003_arducopter_loiter_is_not_automatic_for_mvp() {
        let classification =
            classify_mode(Autopilot::ArduPilot, VehicleFamily::ArduCopter, Some(5));

        assert_eq!(classification, FlightModeClassification::NotAutomatic);
    }

    #[test]
    fn mode_ap_u_004_arducopter_auto_rtl_is_not_automatic_for_mvp() {
        let classification =
            classify_mode(Autopilot::ArduPilot, VehicleFamily::ArduCopter, Some(27));

        assert_eq!(classification, FlightModeClassification::NotAutomatic);
    }

    #[test]
    fn mode_ap_u_005_unsupported_vehicle_is_unknown() {
        let classification = classify_mode(Autopilot::ArduPilot, VehicleFamily::ArduPlane, Some(3));

        assert_eq!(classification, FlightModeClassification::Unknown);
    }

    #[test]
    fn mode_ap_u_006_missing_heartbeat_is_unknown() {
        let classification = classify_mode(Autopilot::ArduPilot, VehicleFamily::ArduCopter, None);

        assert_eq!(classification, FlightModeClassification::Unknown);
    }

    #[test]
    fn mode_px4_u_001_px4_modes_are_unknown_for_limited_scope() {
        let classification = classify_mode(Autopilot::Px4, VehicleFamily::ArduCopter, Some(4));

        assert_eq!(classification, FlightModeClassification::Unknown);
    }

    #[test]
    fn mode_px4_sitl_001_emits_mode_changed_event_without_mode_name() {
        let mut state = FlightState::default();
        let event = state
            .update_heartbeat(HeartbeatObservation {
                autopilot: Autopilot::Px4,
                vehicle_family: VehicleFamily::ArduCopter,
                base_mode: 0,
                custom_mode: Some(4),
                source_system: 1,
                source_component: 1,
            })
            .expect("first PX4 heartbeat changes state");

        assert_eq!(event.audit_event, "flight_state.mode_changed");
        assert_eq!(event.autopilot, Autopilot::Px4);
        assert_eq!(event.classification, FlightModeClassification::Unknown);
        assert_eq!(event.mode_name, None);
    }

    #[test]
    fn mode_ap_sitl_001_emits_mode_changed_event_for_auto() {
        let mut state = FlightState::default();
        let event = state
            .update_heartbeat(HeartbeatObservation {
                autopilot: Autopilot::ArduPilot,
                vehicle_family: VehicleFamily::ArduCopter,
                base_mode: 0,
                custom_mode: Some(3),
                source_system: 1,
                source_component: 1,
            })
            .expect("first heartbeat changes state");

        assert_eq!(event.audit_event, "flight_state.mode_changed");
        assert_eq!(event.classification, FlightModeClassification::Automatic);
        assert_eq!(event.previous_classification, None);
        assert_eq!(event.mode_name, Some("Auto"));
        assert_eq!(event.source_system, 1);
        assert_eq!(event.source_component, 1);
    }

    #[test]
    fn mode_ap_sitl_002_emits_mode_changed_event_when_leaving_auto() {
        let mut state = FlightState::default();
        state.update_heartbeat(HeartbeatObservation {
            autopilot: Autopilot::ArduPilot,
            vehicle_family: VehicleFamily::ArduCopter,
            base_mode: 0,
            custom_mode: Some(3),
            source_system: 1,
            source_component: 1,
        });

        let event = state
            .update_heartbeat(HeartbeatObservation {
                autopilot: Autopilot::ArduPilot,
                vehicle_family: VehicleFamily::ArduCopter,
                base_mode: 0,
                custom_mode: Some(5),
                source_system: 1,
                source_component: 1,
            })
            .expect("mode change must be observable");

        assert_eq!(
            event.previous_classification,
            Some(FlightModeClassification::Automatic)
        );
        assert_eq!(event.classification, FlightModeClassification::NotAutomatic);
        assert_eq!(event.mode_name, Some("Loiter"));
    }

    #[test]
    fn repeated_heartbeat_does_not_emit_mode_changed_event() {
        let mut state = FlightState::default();
        let observation = HeartbeatObservation {
            autopilot: Autopilot::ArduPilot,
            vehicle_family: VehicleFamily::ArduCopter,
            base_mode: 0,
            custom_mode: Some(0),
            source_system: 1,
            source_component: 1,
        };

        assert!(state.update_heartbeat(observation).is_some());
        assert_eq!(state.update_heartbeat(observation), None);
    }
}
