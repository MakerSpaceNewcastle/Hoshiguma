use chrono::{DateTime, TimeDelta, Utc};
use embassy_time::Instant;

pub struct NtpSyncedTime {
    /// Time interval between the device booting and the NTP time sync taking place.
    uptime_at_sync: TimeDelta,

    /// The NTP reported time at `uptime_at_sync`.
    time_at_sync: DateTime<Utc>,
}

impl NtpSyncedTime {
    pub fn new(now: Instant, time_at_sync: DateTime<Utc>) -> Self {
        Self {
            uptime_at_sync: TimeDelta::from_std(now.duration_since(Instant::MIN).into()).unwrap(),
            time_at_sync,
        }
    }

    pub fn now(&self, uptime_now: TimeDelta) -> DateTime<Utc> {
        let uptime_since_time_sync = uptime_now - self.uptime_at_sync;
        self.time_at_sync + uptime_since_time_sync
    }
}
