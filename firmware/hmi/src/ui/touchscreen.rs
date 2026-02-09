//! Touch screen interface.
//! Will only detect the first touch, i.e. no dragging, but that is good enough.

use super::UI_TOUCH_POINT;
use crate::{Notification, api::NOTIFICATIONS};
use defmt::{Format, debug, info, warn};
use embassy_time::{Duration, Timer, with_timeout};
use embedded_graphics::{
    Drawable,
    pixelcolor::Rgb666,
    prelude::{Point, Primitive, RgbColor, Size},
    primitives::{PrimitiveStyle, Rectangle},
};
use heapless::Vec;
use peek_o_display_bsp::{display::Display, embassy_rp::gpio::Input, touch::Touch};

#[embassy_executor::task]
pub(super) async fn task(mut touch: Touch, mut irq: Input<'static>) {
    let ui_touch_point_tx = UI_TOUCH_POINT.sender();

    touch.read();

    let mut touch_count = 0usize;

    loop {
        // Wait for a touch
        irq.wait_for_low().await;
        touch_count = touch_count.saturating_add(1);

        // Average multiple measurements to get a point
        let mut measurements = Vec::<_, 20>::new();
        for _ in 0..measurements.capacity() {
            let point = touch.read();
            debug!("Touch measurement: {}", point);
            measurements.push(point).unwrap();
            Timer::after_millis(2).await;
        }
        let point = average_measurements(&measurements);

        // Notify panel interaction
        if with_timeout(
            Duration::from_millis(100),
            NOTIFICATIONS.send(Notification::PanelInteraction),
        )
        .await
        .is_err()
        {
            warn!("Timeout when sending panel interaction notification");
        }

        info!("touch {} at {:?}", touch_count, point);

        if let Some((x, y)) = point {
            let point = Point::new(x, y);
            let button = detect_soft_button(point);
            ui_touch_point_tx.send((point, button));
        }

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

    for (x, y) in measurement.iter().flatten() {
        sum_x += x;
        sum_y += y;
        count += 1;
    }
    debug!("Averaged {} measurements", count);

    if count > 0 {
        Some((sum_x / count, sum_y / count))
    } else {
        None
    }
}

#[derive(Debug, Format, Copy, Clone, PartialEq, Eq)]
pub(super) enum SoftButton {
    A,
    B,
    C,
}

fn detect_soft_button(point: Point) -> Option<SoftButton> {
    if BUTTON_A_TOUCH_BOX.contains(point) {
        Some(SoftButton::A)
    } else if BUTTON_B_TOUCH_BOX.contains(point) {
        Some(SoftButton::B)
    } else if BUTTON_C_TOUCH_BOX.contains(point) {
        Some(SoftButton::C)
    } else {
        None
    }
}

const BUTTON_A_TOUCH_BOX: Rectangle =
    Rectangle::with_center(Point::new(40, 290), Size::new(60, 40));
const BUTTON_B_TOUCH_BOX: Rectangle =
    Rectangle::with_center(Point::new(120, 290), Size::new(60, 40));
const BUTTON_C_TOUCH_BOX: Rectangle =
    Rectangle::with_center(Point::new(200, 290), Size::new(60, 40));

pub(super) fn draw_soft_button_touch_boxes(display: &mut Display) {
    let b = BUTTON_A_TOUCH_BOX.into_styled(PrimitiveStyle::with_fill(Rgb666::RED));
    b.draw(display).unwrap();

    let b = BUTTON_B_TOUCH_BOX.into_styled(PrimitiveStyle::with_fill(Rgb666::GREEN));
    b.draw(display).unwrap();

    let b = BUTTON_C_TOUCH_BOX.into_styled(PrimitiveStyle::with_fill(Rgb666::BLUE));
    b.draw(display).unwrap();
}
