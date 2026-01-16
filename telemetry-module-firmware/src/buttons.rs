use defmt::{Format, debug, info};
use embassy_rp::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Instant, Timer};

#[derive(Clone, PartialEq, Eq, Format)]
pub(crate) enum UiEvent {
    ButtonPushed,
    ButtonPushedForALongTime,
}

pub(crate) static UI_INPUTS: PubSubChannel<CriticalSectionRawMutex, UiEvent, 8, 2, 1> =
    PubSubChannel::new();

const PUSH_THRESHOLD: Duration = Duration::from_millis(75);
const LONG_PUSH_THRESHOLD: Duration = Duration::from_secs(5);

#[embassy_executor::task]
pub(super) async fn task(r: crate::ButtonResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("ui button").await;

    let mut button = Input::new(r.a_pin, Pull::Up);
    let tx = UI_INPUTS.publisher().unwrap();

    loop {
        button.wait_for_low().await;
        let time_down = Instant::now();

        button.wait_for_high().await;
        let time_up = Instant::now();

        let time_pressed_for = time_up - time_down;
        debug!("Button was down for {}ms", time_pressed_for.as_millis());

        if time_pressed_for > LONG_PUSH_THRESHOLD {
            info!("Button pressed for a long time");
            tx.publish(UiEvent::ButtonPushedForALongTime).await;
        } else if time_pressed_for > PUSH_THRESHOLD {
            info!("Button pressed");
            tx.publish(UiEvent::ButtonPushed).await;
        } else {
            continue;
        }

        // Wait a little while before allowing further pushes
        Timer::after_millis(250).await;
    }
}
