use crate::HeaderTankLevelSensorResources;
use embassy_rp::gpio::{Input, Level, Pull};
use hoshiguma_protocol::cooler::types::{HeaderTankCoolantLevel, HeaderTankCoolantLevelReading};

pub(crate) struct HeaderTankLevelSensor {
    empty: Input<'static>,
    low: Input<'static>,
}

impl HeaderTankLevelSensor {
    pub(crate) fn new(r: HeaderTankLevelSensorResources) -> Self {
        let empty = Input::new(r.empty, Pull::Down);
        let low = Input::new(r.low, Pull::Down);
        Self { empty, low }
    }

    pub(crate) fn get(&self) -> HeaderTankCoolantLevelReading {
        match (self.empty.get_level(), self.low.get_level()) {
            (Level::Low, Level::Low) => Ok(HeaderTankCoolantLevel::Full),
            (Level::Low, Level::High) => Err(()),
            (Level::High, Level::Low) => Ok(HeaderTankCoolantLevel::Normal),
            (Level::High, Level::High) => Ok(HeaderTankCoolantLevel::Empty),
        }
    }
}
