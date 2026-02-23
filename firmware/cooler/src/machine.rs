use crate::devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    radiator_fan::RadiatorFan, temperature_sensors::TemperatureSensors,
};
use hoshiguma_api::cooler::{CompressorState, CoolantPumpState, RadiatorFanState};

pub(crate) struct Machine {
    pub coolant_pump: CoolantPump,
    pub compressor: Compressor,
    pub radiator_fan: RadiatorFan,

    pub coolant_flow_sensor: CoolantFlowSensor,
    pub temperature_sensors: TemperatureSensors,
}

impl Machine {
    pub(crate) fn set_off(&mut self) {
        self.coolant_pump.set(CoolantPumpState::Idle);
        self.compressor.set(CompressorState::Idle);
        self.radiator_fan.set(RadiatorFanState::Idle);
    }
}
