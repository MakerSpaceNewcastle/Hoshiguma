pub(crate) mod gpio_relays;

use crate::logic::{
    air_assist::AirAssistStatus,
    extraction::ExtractionStatus,
    machine::{MachineProblem, MachineStatus},
    AlarmState, StatusLight,
};
use ufmt::derive::uDebug;

#[derive(uDebug, PartialEq)]
pub(crate) struct Outputs {
    pub controller_machine_alarm: AlarmState,
    pub controller_cooling_alarm: AlarmState,
    pub laser_enable: bool,
    pub status_light: StatusLight,
    pub air_pump: bool,
    pub extractor_fan: bool,
}

impl Outputs {
    pub(crate) fn new(
        machine: &MachineStatus,
        extraction: &ExtractionStatus,
        air_assist: &AirAssistStatus,
    ) -> Self {
        match machine {
            MachineStatus::Running => Outputs {
                controller_machine_alarm: AlarmState::Normal,
                controller_cooling_alarm: AlarmState::Normal,
                laser_enable: true,
                status_light: StatusLight::Amber,
                air_pump: air_assist.active(),
                extractor_fan: extraction.active(),
            },
            MachineStatus::Idle => Outputs {
                controller_machine_alarm: AlarmState::Normal,
                controller_cooling_alarm: AlarmState::Normal,
                laser_enable: true,
                status_light: StatusLight::Green,
                air_pump: air_assist.active(),
                extractor_fan: extraction.active(),
            },
            MachineStatus::Problem(problem) => Outputs {
                controller_machine_alarm: if *problem == MachineProblem::DoorOpen {
                    AlarmState::Alarm
                } else {
                    AlarmState::Normal
                },
                controller_cooling_alarm: if *problem == MachineProblem::CoolingFault {
                    AlarmState::Alarm
                } else {
                    AlarmState::Normal
                },
                laser_enable: false,
                status_light: StatusLight::Red,
                air_pump: air_assist.active(),
                extractor_fan: extraction.active(),
            },
        }
    }
}

pub(crate) trait WriteOutputs {
    fn write(&mut self, outputs: &Outputs);
}
