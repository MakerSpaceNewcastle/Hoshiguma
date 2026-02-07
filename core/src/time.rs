pub struct NtpSyncedTime {
    /// Time in microseconds since the device has booted, at the point when NTP time sync took place.
    device_uptime_at_sync: u64,

    /// Clock offset in microseconds between `device_uptime_at_sync` and UNIX epoch NTP time.
    ntp_offset: u64,
}

impl NtpSyncedTime {
    pub fn new(device_uptime_at_sync: u64, ntp_offset: i64) -> Self {
        let ntp_offset = ntp_offset
            .try_into()
            .expect("NTP offset should be positive, a negative offset does not make sense");

        Self {
            device_uptime_at_sync,
            ntp_offset,
        }
    }

    pub fn unix(&self, device_uptime_now: u64) -> u64 {
        self.time_at_sync() + self.age(device_uptime_now)
    }

    pub fn time_at_sync(&self) -> u64 {
        self.device_uptime_at_sync.saturating_add(self.ntp_offset)
    }

    pub fn age(&self, device_uptime_now: u64) -> u64 {
        device_uptime_now - self.device_uptime_at_sync
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[should_panic]
    fn negative_offset() {
        let _ = NtpSyncedTime::new(0, -42);
    }

    #[test]
    fn create_zero() {
        let _ = NtpSyncedTime::new(0, 0);
    }

    #[test]
    fn create() {
        let _ = NtpSyncedTime::new(0, 42);
    }
}
