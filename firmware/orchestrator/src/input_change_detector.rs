use defmt::debug;
use embassy_rp::gpio::{Input, Level};

pub(crate) struct InputChangeDetector {
    pin: Input<'static>,
    last_level: Option<Level>,
}

impl InputChangeDetector {
    pub(crate) fn new(pin: Input<'static>) -> Self {
        Self {
            pin,
            last_level: None,
        }
    }

    pub(crate) async fn wait_for_change(&mut self) -> Level {
        let level_now = self.pin.get_level();

        if let Some(level) = self.handle_level_change(level_now) {
            return level;
        } else {
            let level_now = match level_now {
                Level::Low => {
                    self.pin.wait_for_high().await;
                    Level::High
                }
                Level::High => {
                    self.pin.wait_for_low().await;
                    Level::Low
                }
            };

            if let Some(level) = self.handle_level_change(level_now) {
                return level;
            }
        }

        unreachable!()
    }

    fn handle_level_change(&mut self, level_now: Level) -> Option<Level> {
        if self.last_level != Some(level_now) {
            debug!("Detected pin change, now {}", level_now);
            self.last_level = Some(level_now);
            Some(level_now)
        } else {
            None
        }
    }
}
