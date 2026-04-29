use crate::api::NUM_LISTENERS;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Timer};
use hoshiguma_api::telemetry_bridge::FormattedTelemetryDataPoint;

const TELEMETRY_PUBLISHERS: usize = NUM_LISTENERS + 1;

pub(crate) static TELEMETRY_TX: PubSubChannel<
    CriticalSectionRawMutex,
    FormattedTelemetryDataPoint,
    32,
    1,
    TELEMETRY_PUBLISHERS,
> = PubSubChannel::new();

#[embassy_executor::task]
pub(super) async fn task() {
    // TODO
    loop {
        Timer::after(Duration::from_secs(1)).await;
    }
}
