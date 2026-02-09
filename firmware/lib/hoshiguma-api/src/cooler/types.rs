use core::{ops::Sub, time::Duration};
use defmt::Format;
use getset::Getters;
use nutype::nutype;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CoolantPumpState {
    Idle,
    Run,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum CompressorState {
    Idle,
    Run,
}

#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum RadiatorFanState {
    Idle,
    Run,
}

/// The raw coolant rate measured in encoder pulses per unit time.
#[derive(Debug, Format, Clone, Copy, PartialEq, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct RawCoolantRate {
    pulses: u16,
    time: Duration,
}

impl RawCoolantRate {
    pub fn new(pulses: u16, time: Duration) -> Self {
        Self { pulses, time }
    }

    pub fn into_rate(self, pulses_per_litre: f64) -> CoolantRate {
        let litres = (self.pulses as f64) / pulses_per_litre;
        let seconds = self.time.as_secs() as f64;
        let litres_per_minute = (litres / seconds) * 60.0;
        CoolantRate::new(litres_per_minute)
    }
}

/// The rate of flow of coolant in litres per minute.
#[nutype(
    const_fn,
    default = 0.0,
    derive(
        Default,
        Debug,
        Clone,
        Copy,
        PartialEq,
        PartialOrd,
        Serialize,
        Deserialize,
    ),
    derive_unchecked(Format)
)]
pub struct CoolantRate(f64);

impl Sub for CoolantRate {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.into_inner() - rhs.into_inner())
    }
}
