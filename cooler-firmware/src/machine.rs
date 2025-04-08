use crate::devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    header_tank_level_sensor::HeaderTankLevelSensor,
    heat_exchanger_level_sensor::HeatExchangerLevelSensor, radiator_fan::RadiatorFan,
    stirrer::Stirrer, temperature_sensors::TemperatureSensors,
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
