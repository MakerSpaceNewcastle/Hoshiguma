use defmt::{debug, info};
use embassy_rp::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Timer};

pub(crate) enum UiEvent {
    ButtonPushed,
}

pub(crate) static UI_INPUTS: Channel<CriticalSectionRawMutex, UiEvent, 16> = Channel::new();

const PUSH_THRESHOLD: Duration = Duration::from_millis(75);

#[embassy_executor::task]
pub(super) async fn task(r: crate::UiResources) {
    let mut button = Input::new(r.button, Pull::Up);

    loop {
        button.wait_for_low().await;
        let time_down = Instant::now();

        button.wait_for_high().await;
        let time_up = Instant::now();

        let time_pressed_for = time_up - time_down;
        debug!("Button was down for {}ms", time_pressed_for.as_millis());

        if time_pressed_for > PUSH_THRESHOLD {
            info!("Button pressed");
            UI_INPUTS.send(UiEvent::ButtonPushed).await;

            // Wait a little while before allowing further pushes
            Timer::after_millis(250).await;
        }
    }
}
