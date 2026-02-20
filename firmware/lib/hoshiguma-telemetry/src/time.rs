use chrono::{DateTime, TimeDelta, Utc};

pub struct NtpSyncedTime {
    /// Time interval between the device booting and the NTP time sync taking place.
    uptime_at_sync: TimeDelta,

    /// The NTP reported time at `uptime_at_sync`.
    time_at_sync: DateTime<Utc>,
}

impl NtpSyncedTime {
    pub fn new(uptime_at_sync: TimeDelta, time_at_sync: DateTime<Utc>) -> Self {
        Self {
            uptime_at_sync,
            time_at_sync,
        }
    }

    pub fn now(&self, uptime_now: TimeDelta) -> DateTime<Utc> {
        let uptime_since_time_sync = uptime_now - self.uptime_at_sync;
        self.time_at_sync + uptime_since_time_sync
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn time_from_sync_record() {
        let t = NaiveDate::from_ymd_opt(2026, 2, 7)
            .unwrap()
            .and_hms_opt(11, 39, 40)
            .unwrap()
            .and_utc();

        let uptime_at_sync = TimeDelta::milliseconds(15_000);

        let time = NtpSyncedTime::new(uptime_at_sync, t);

        assert_eq!(
            time.now(TimeDelta::milliseconds(30_000)),
            NaiveDate::from_ymd_opt(2026, 2, 7)
                .unwrap()
                .and_hms_opt(11, 39, 55)
                .unwrap()
                .and_utc()
        )
    }
}
