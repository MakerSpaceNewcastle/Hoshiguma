use crate::devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    coolant_reservoir_level_sensor::CoolantReservoirLevelSensor, radiator_fan::RadiatorFan,
    temperature_sensors::TemperatureSensors,
};
use hoshiguma_core::accessories::cooler::types::{
    CompressorState, CoolantPumpState, RadiatorFanState, State,
};

pub(crate) struct Machine {
    pub coolant_pump: CoolantPump,
    pub compressor: Compressor,
    pub radiator_fan: RadiatorFan,

    pub coolant_reservoir_level_sensor: CoolantReservoirLevelSensor,
    pub coolant_flow_sensor: CoolantFlowSensor,
    pub temperature_sensors: TemperatureSensors,
}

impl Machine {
    pub(crate) async fn state(&mut self) -> State {
        State {
            coolant_pump: self.coolant_pump.get(),
            compressor: self.compressor.get(),
            radiator_fan: self.radiator_fan.get(),

            coolant_reservoir_level: self.coolant_reservoir_level_sensor.get(),
            coolant_flow_rate: self.coolant_flow_sensor.get().await,
            temperatures: self.temperature_sensors.get().await,
        }
    }

    pub(crate) fn set_off(&mut self) {
        self.coolant_pump.set(CoolantPumpState::Idle);
        self.compressor.set(CompressorState::Idle);
        self.radiator_fan.set(RadiatorFanState::Idle);
    }
}
