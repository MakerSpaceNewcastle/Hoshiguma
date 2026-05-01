use crate::telemetry_bridge_comm::get_time_from_telemetry_bridge;
use chrono::{DateTime, Utc};
use core::sync::atomic::Ordering;
use defmt::{debug, info};
use embassy_net::Stack;
use embassy_time::{Duration, Instant, Timer};
use portable_atomic::AtomicI64;

/// Offset in microseconds to add to uptime to get world time.
static BOOT_CLOCK_WALL_OFFSET_US: AtomicI64 = AtomicI64::new(0);

/// Gets the current wall time.
///
/// Time is only valid after a successful NTP sync.
pub(crate) fn now() -> Option<DateTime<Utc>> {
    let now = Instant::now().as_micros() as i64;
    let offset = BOOT_CLOCK_WALL_OFFSET_US.load(Ordering::Relaxed);
    match offset {
        0 => None,
        _ => Some(DateTime::from_timestamp_micros(now + offset).unwrap()),
    }
}

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) -> ! {
    let mut last_sync = None;

    loop {
        if let Ok(time) = get_time_from_telemetry_bridge(stack).await {
            let now = Instant::now();
            let now_us = now.as_micros() as i64;

            let time = time.timestamp_micros();

            let offset = time - now_us;
            info!(
                "Got time from telemetry bridge: {}, our clock: {} us, offset: {} us",
                time, now, offset
            );

            BOOT_CLOCK_WALL_OFFSET_US.store(offset, Ordering::Relaxed);
            info!("Time now: {}", self::now());

            last_sync = Some(now);
        }

        let update_interval = match last_sync {
            Some(last_sync) => {
                if (Instant::now() - last_sync) >= Duration::from_secs(60) {
                    Duration::from_secs(10)
                } else {
                    Duration::from_secs(60)
                }
            }
            None => Duration::from_secs(10),
        };

        debug!(
            "Waiting {}s before next time sync",
            update_interval.as_secs()
        );
        Timer::after(update_interval).await;
    }
}
