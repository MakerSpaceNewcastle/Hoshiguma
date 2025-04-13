use crate::devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    header_tank_level_sensor::HeaderTankLevelSensor,
    heat_exchanger_level_sensor::HeatExchangerLevelSensor, radiator_fan::RadiatorFan,
    stirrer::Stirrer, temperature_sensors::TemperatureSensors,
};
use hoshiguma_protocol::cooler::types::{
    CompressorState, CoolantPumpState, RadiatorFanState, State, StirrerState,
};

pub(crate) struct Machine {
    pub stirrer: Stirrer,
    pub coolant_pump: CoolantPump,
    pub compressor: Compressor,
    pub radiator_fan: RadiatorFan,

    pub header_tank_level: HeaderTankLevelSensor,
    pub heat_exchanger_level: HeatExchangerLevelSensor,
    pub coolant_flow_sensor: CoolantFlowSensor,
    pub temperature_sensors: TemperatureSensors,
}

impl Machine {
    pub(crate) async fn state(&mut self) -> State {
        State {
            stirrer: self.stirrer.get(),
            coolant_pump: self.coolant_pump.get(),
            compressor: self.compressor.get(),
            radiator_fan: self.radiator_fan.get(),

            coolant_header_tank_level: self.header_tank_level.get(),
            heat_exchange_fluid_level: self.heat_exchanger_level.get(),
            coolant_flow_rate: self.coolant_flow_sensor.get().await,
            temperatures: self.temperature_sensors.get().await,
        }
    }

    pub(crate) fn set_off(&mut self) {
        self.stirrer.set(StirrerState::Idle);
        self.coolant_pump.set(CoolantPumpState::Idle);
        self.compressor.set(CompressorState::Idle);
        self.radiator_fan.set(RadiatorFanState::Idle);
    }
}
