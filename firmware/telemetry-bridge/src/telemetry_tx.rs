use crate::api::NUM_LISTENERS;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use hoshiguma_api::telemetry_bridge::FormattedTelemetryDataPoint;

const TELEMETRY_PUBLISHERS: usize = NUM_LISTENERS + 1;

pub(crate) static TELEMETRY_TX: PubSubChannel<
    CriticalSectionRawMutex,
    FormattedTelemetryDataPoint,
    32,
    1,
    TELEMETRY_PUBLISHERS,
> = PubSubChannel::new();

// TODO
