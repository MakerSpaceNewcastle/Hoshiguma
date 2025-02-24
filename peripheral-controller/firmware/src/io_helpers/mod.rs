use defmt::info;
use embassy_rp::gpio::{Input, Level};
use embassy_time::{Duration, Ticker};

pub(crate) struct PolledInput {
    pin: Input<'static>,
    last_level: Option<Level>,
    poll_interval: Duration,
}

impl PolledInput {
    pub(crate) fn new(pin: Input<'static>, poll_interval: Duration) -> Self {
        Self {
            pin,
            last_level: None,
            poll_interval,
        }
    }

    pub(crate) async fn wait_for_change(&mut self) -> Level {
        let mut tick = Ticker::every(self.poll_interval);

        loop {
            tick.next().await;

            let new_level = self.pin.get_level();

            if self.last_level != Some(new_level) {
                info!("Detected pin change");
                self.last_level = Some(new_level);
                return new_level;
            }
        }
    }

    pub(crate) async fn level(&mut self) -> Level {
        if self.last_level.is_none() {
            self.last_level = Some(self.pin.get_level());
        }

        self.last_level.unwrap()
    }
}
