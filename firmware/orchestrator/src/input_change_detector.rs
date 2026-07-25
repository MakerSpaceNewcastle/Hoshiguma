use defmt::debug;
use embassy_rp::gpio::{Input, Level};
use embassy_time::{Duration, Timer};

pub(crate) struct InputChangeDetector {
    pin: Input<'static>,
    last_level: Option<Level>,
    low_debounce: Duration,
    high_debounce: Duration,
}

impl InputChangeDetector {
    pub(crate) fn new(
        pin: Input<'static>,
        low_debounce: Duration,
        high_debounce: Duration,
    ) -> Self {
        Self {
            pin,
            last_level: None,
            low_debounce,
            high_debounce,
        }
    }

    pub(crate) async fn wait_for_change(&mut self) -> Level {
        loop {
            let level_now = self.pin.get_level();

            if self.last_level != Some(level_now) {
                Timer::after(match level_now {
                    Level::Low => self.low_debounce,
                    Level::High => self.high_debounce,
                })
                .await;

                if self.pin.get_level() == level_now {
                    debug!("Detected pin change, now {}", level_now);
                    self.last_level = Some(level_now);
                    return level_now;
                }
            }

            match self.pin.get_level() {
                Level::Low => self.pin.wait_for_high().await,
                Level::High => self.pin.wait_for_low().await,
            }
        }
    }
}
