//! Touch screen interface.
//! Will only detect the first touch, i.e. no dragging, but that is good enough.

use defmt::{debug, info};
use embassy_time::Timer;
use heapless::Vec;
use peek_o_display_bsp::{embassy_rp::gpio::Input, touch::Touch};

#[embassy_executor::task]
pub(crate) async fn task(mut touch: Touch, mut irq: Input<'static>) {
    touch.read();

    let mut touch_count = 0usize;

    loop {
        // Wait for a touch
        irq.wait_for_low().await;
        touch_count = touch_count.saturating_add(1);

        let mut measurements = Vec::<_, 20>::new();
        for _ in 0..measurements.capacity() {
            let point = touch.read();
            debug!("Touch measurement: {}", point);
            measurements.push(point).unwrap();
            Timer::after_millis(2).await;
        }
        let point = average_measurements(&measurements);

        // TODO
        info!("touch {} at {:?}", touch_count, point);

        // Wait for the touch to end
        irq.wait_for_high().await;

        // Small delay to debounce the touch end
        Timer::after_millis(200).await;
    }
}

fn average_measurements(measurement: &[Option<(i32, i32)>]) -> Option<(i32, i32)> {
    let mut sum_x = 0;
    let mut sum_y = 0;
    let mut count = 0;

    for m in measurement {
        if let Some((x, y)) = m {
            sum_x += x;
            sum_y += y;
            count += 1;
        }
    }
    debug!("Averaged {} measurements", count);

    if count > 0 {
        Some((sum_x / count, sum_y / count))
    } else {
        None
    }
}
