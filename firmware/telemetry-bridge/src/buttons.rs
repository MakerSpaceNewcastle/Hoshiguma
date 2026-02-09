use crate::ButtonResources;
use defmt::{Format, debug};
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Instant, Timer};

pub(crate) static BUTTON_EVENTS: PubSubChannel<CriticalSectionRawMutex, ButtonEvent, 8, 1, 3> =
    PubSubChannel::new();

#[derive(Format, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ButtonEvent {
    Pressed(Button),
    Released(Button, Option<Duration>),
}

#[derive(Format, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Button {
    One,
    Two,
    Three,
}

pub(crate) fn init(r: ButtonResources, spawner: Spawner) {
    let btn_1 = Input::new(r.user_1, Pull::Up);
    let btn_2 = Input::new(r.user_2, Pull::Up);
    let btn_3 = Input::new(r.user_3, Pull::Up);

    spawner.spawn(button_task(btn_1, Button::One).unwrap());
    spawner.spawn(button_task(btn_2, Button::Two).unwrap());
    spawner.spawn(button_task(btn_3, Button::Three).unwrap());
}

#[embassy_executor::task(pool_size = 3)]
async fn button_task(mut pin: Input<'static>, button: Button) -> ! {
    let publisher = BUTTON_EVENTS.publisher().unwrap();

    // Start by just waiting for the button to be released
    pin.wait_for_high().await;
    publisher.publish(ButtonEvent::Released(button, None)).await;

    loop {
        // Wait for the button to be pressed
        pin.wait_for_low().await;
        let press_time = Instant::now();
        let event = ButtonEvent::Pressed(button);
        debug!("{}", event);
        publisher.publish(event).await;

        // Wait for a little bit (as a crude debounce)
        Timer::after_millis(10).await;

        // Then wait for it to be released
        pin.wait_for_high().await;
        let release_time = Instant::now();
        let event = ButtonEvent::Released(button, Some(release_time - press_time));
        debug!("{}", event);
        publisher.publish(event).await;

        // Wait for a little bit (as a crude debounce)
        Timer::after_millis(10).await;
    }
}
