use crate::HeatExchangerLevelSensorResources;
use embassy_rp::gpio::{Input, Level, Pull};
use hoshiguma_protocol::cooler::types::HeatExchangeFluidLevel;

pub(crate) struct HeatExchangerLevelSensor {
    low: Input<'static>,
}

impl HeatExchangerLevelSensor {
    pub(crate) fn new(r: HeatExchangerLevelSensorResources) -> Self {
        let low = Input::new(r.low, Pull::Down);
        Self { low }
    }

    pub(crate) fn get(&self) -> HeatExchangeFluidLevel {
        match self.low.get_level() {
            Level::Low => HeatExchangeFluidLevel::Normal,
            Level::High => HeatExchangeFluidLevel::Low,
        }
    }
}
