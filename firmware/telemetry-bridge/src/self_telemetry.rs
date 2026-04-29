//! Produces data points for the telemetry module itself.
//!
//! Sent on boot:
//! - firmware Git revision
//! - boot reason
//!
//! Sent at a 1 minute interval:
//! - uptime
//! - data points inserted into buffer
//! - data points discarded due to buffer overflow
//! - data submission success counter
//! - data submission failure counter

use crate::telemetry_tx::TELEMETRY_TX;
use core::sync::atomic::Ordering;
use defmt::warn;
use embassy_time::{Instant, Timer};
use hoshiguma_api::{TelemetryString, telemetry_bridge::FormattedTelemetryDataPoint};
use hoshiguma_common::telemetry::{format_influx_line, format_influx_line_str};
use portable_atomic::{AtomicU64, AtomicUsize};

pub(crate) static DATA_POINTS_ACCEPTED: AtomicUsize = AtomicUsize::new(0);
pub(crate) static DATA_POINTS_DISCARDED: AtomicUsize = AtomicUsize::new(0);

pub(crate) static TELEGRAF_SUBMIT_SUCCESS: AtomicUsize = AtomicUsize::new(0);
pub(crate) static TELEGRAF_SUBMIT_FAIL: AtomicUsize = AtomicUsize::new(0);

pub(crate) static STRING_REGISTRY_LAST_MODIFIED: AtomicU64 = AtomicU64::new(0);
pub(crate) static STRING_REGISTRY_SIZE: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn string_registry_last_modification_age_ms() -> u64 {
    Instant::now()
        .as_millis()
        .saturating_sub(STRING_REGISTRY_LAST_MODIFIED.load(Ordering::Relaxed))
}

#[embassy_executor::task]
pub(crate) async fn task() {
    let telem_tx = TELEMETRY_TX.publisher().unwrap();

    macro_rules! publish_datapoint {
        ($datapoint:expr) => {
            match $datapoint {
                Ok(line) => {
                    telem_tx.publish_immediate(FormattedTelemetryDataPoint(line));
                }
                Err(_) => {
                    warn!("Failed to format data point");
                }
            };
        };
    }

    // Send data points that only change on boot
    {
        publish_datapoint!(format_influx_line_str(
            "telemetry_module_git_revision",
            "value",
            git_version::git_version!(),
            None,
        ));

        publish_datapoint!(format_influx_line_str(
            "telemetry_module_boot_reason",
            "value",
            crate::boot_reason().telemetry_str(),
            None,
        ));
    }

    loop {
        publish_datapoint!(format_influx_line(
            "telemetry_module_uptime",
            "value",
            Instant::now().as_millis(),
            None,
        ));

        publish_datapoint!(format_influx_line(
            "telemetry_module_data_points,state=accepted",
            "count",
            DATA_POINTS_ACCEPTED.load(Ordering::Relaxed),
            None
        ));

        publish_datapoint!(format_influx_line(
            "telemetry_module_data_points,state=discarded",
            "count",
            DATA_POINTS_DISCARDED.load(Ordering::Relaxed),
            None
        ));

        publish_datapoint!(format_influx_line(
            "telemetry_module_data_submissions,result=success",
            "count",
            TELEGRAF_SUBMIT_SUCCESS.load(Ordering::Relaxed),
            None
        ));

        publish_datapoint!(format_influx_line(
            "telemetry_module_data_submissions,result=fail",
            "count",
            TELEGRAF_SUBMIT_FAIL.load(Ordering::Relaxed),
            None
        ));

        // Send data points (approximately) every minute
        Timer::after_secs(60).await;
    }
}
