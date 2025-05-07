use crate::CoolantReservoirLevelSensorResources;
use embassy_rp::gpio::{Input, Level, Pull};
use hoshiguma_protocol::cooler::types::CoolantReservoirLevel;

pub(crate) struct CoolantReservoirLevelSensor {
    low: Input<'static>,
}

impl CoolantReservoirLevelSensor {
    pub(crate) fn new(r: CoolantReservoirLevelSensorResources) -> Self {
        let low = Input::new(r.low, Pull::Down);
        Self { low }
    }

    pub(crate) fn get(&self) -> CoolantReservoirLevel {
        match self.low.get_level() {
            Level::Low => CoolantReservoirLevel::Normal,
            Level::High => CoolantReservoirLevel::Low,
        }
    }
}
