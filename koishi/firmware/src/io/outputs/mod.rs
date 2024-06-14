pub(crate) mod gpio_relays;

use crate::logic::{air_assist::AirAssistStatusExt, extraction::ExtractionStatusExt};
use hoshiguma_foundational_data::koishi::{
    AirAssistStatus, AlarmState, ExtractionStatus, MachineStatus, Outputs, StatusLight,
};

pub(crate) trait OutputsExt {
    fn new(
        machine: &MachineStatus,
        extraction: &ExtractionStatus,
        air_assist: &AirAssistStatus,
    ) -> Self;
}

impl OutputsExt for Outputs {
    fn new(
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
            MachineStatus::Problem(_) => Outputs {
                controller_machine_alarm: AlarmState::Alarm,
                controller_cooling_alarm: AlarmState::Normal,
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
