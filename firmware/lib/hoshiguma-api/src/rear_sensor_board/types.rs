use core::time::Duration;
use defmt::Format;
use serde::{Deserialize, Serialize};

pub const NUM_ONEWIRE_TEMPERATURE_SENSORS: usize = 8;

pub type OnewireTemperatureSensorReadings =
    crate::OnewireTemperatureSensorReadings<NUM_ONEWIRE_TEMPERATURE_SENSORS>;

pub type AirflowSensorMeasurement = Result<AirflowSensorMeasurementInner, ()>;

#[derive(Debug, Format, Clone, PartialEq, Serialize, Deserialize)]
pub struct AirflowSensorMeasurementInner {
    pub differential_pressure: f32,
    pub temperature: f32,
}

#[derive(Debug, Format, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatusLightSettings {
    pub red: LightPattern,
    pub amber: LightPattern,
    pub green: LightPattern,
}

/// Represents a repeating pattern of binary light states.
#[derive(Debug, Format, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct LightPattern(pub [LightState; Self::NUM_STEPS]);

impl LightPattern {
    pub const NUM_STEPS: usize = 20;
    pub const STEP_DURATION: Duration = Duration::from_millis(100);
    pub const SEQUENCE_DURATION: Duration =
        Duration::from_millis((Self::STEP_DURATION.as_millis() * Self::NUM_STEPS as u128) as u64);

    /// Returns the light state at the given time, based on the pattern.
    /// `time` is the duration since the start of the sequence and is allowed to exceed the total sequence duration, in which case it will wrap.
    pub fn state_at_time(&self, time: Duration) -> LightState {
        let step =
            ((time.as_millis() / Self::STEP_DURATION.as_millis()) as usize) % Self::NUM_STEPS;
        self.0[step]
    }

    pub const ON: Self = Self([LightState::On; Self::NUM_STEPS]);
    pub const OFF: Self = Self([LightState::Off; Self::NUM_STEPS]);

    pub const BLINK_1HZ: Self = Self([
        LightState::On, // 0 (0s)
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::Off, // 10 (1s)
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
    ]);
    pub const BLINK_2HZ: Self = Self([
        LightState::On, // 0 (0s)
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::Off, // 5 (0.5s)
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::On, // 10 (1s)
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::On,
        LightState::Off, // 15 (1.5s)
        LightState::Off,
        LightState::Off,
        LightState::Off,
        LightState::Off,
    ]);
}

#[derive(Debug, Format, Default, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum LightState {
    #[default]
    Off,
    On,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_pattern_state_at_time() {
        let pattern = LightPattern::BLINK_2HZ;

        assert_eq!(
            pattern.state_at_time(Duration::from_millis(0)),
            LightState::On
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(499)),
            LightState::On
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(500)),
            LightState::Off
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(1000)),
            LightState::On
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(1500)),
            LightState::Off
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(2000)),
            LightState::On
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(2499)),
            LightState::On
        );
        assert_eq!(
            pattern.state_at_time(Duration::from_millis(2500)),
            LightState::Off
        );
    }
}
