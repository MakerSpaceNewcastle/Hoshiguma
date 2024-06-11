use crate::status::CoolantLevel;
use embedded_hal::digital::InputPin;

pub(crate) struct CoolantLevelSensor<PH: InputPin, PL: InputPin> {
    top_float_switch: PH,
    bottom_float_switch: PL,
}

impl<PH: InputPin, PL: InputPin> CoolantLevelSensor<PH, PL> {
    pub(crate) fn new(top_float_switch: PH, bottom_float_switch: PL) -> Self {
        Self {
            top_float_switch,
            bottom_float_switch,
        }
    }

    pub(crate) fn read(&mut self) -> Option<CoolantLevel> {
        // If the sensor is submerged then the float is lifted up, closing the switch, hence the
        // pin reads low.
        let top_submerged = self.top_float_switch.is_low();
        let bottom_submerged = self.bottom_float_switch.is_low();

        if let Ok(top_submerged) = top_submerged {
            if let Ok(bottom_submerged) = bottom_submerged {
                return match (top_submerged, bottom_submerged) {
                    // Both level switches are under water
                    (true, true) => Some(CoolantLevel::Full),
                    // Only the top level switch is under water (something's fucky...)
                    (true, false) => None,
                    // Only the bottom level switch is under water
                    (false, true) => Some(CoolantLevel::Low),
                    // Neither level switch is under water
                    (false, false) => Some(CoolantLevel::CriticallyLow),
                };
            }
        }

        None
    }
}
