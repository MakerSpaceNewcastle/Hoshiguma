use defmt::info;
use embassy_futures::select::{Either3, select3};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, watch::Watch};
use embassy_time::{Duration, Instant};
use hoshiguma_api::hmi::BacklightMode;
use hoshiguma_common::maybe_timer::MaybeTimer;
use peek_o_display_bsp::embassy_rp::gpio::Output;

pub(crate) static BACKLIGHT_MODE: Watch<CriticalSectionRawMutex, BacklightMode, 1> = Watch::new();
pub(crate) static BACKLIGHT_WAKE: Channel<CriticalSectionRawMutex, (), 1> = Channel::new();
pub(super) static BACKLIGHT_ON: Watch<CriticalSectionRawMutex, bool, 1> = Watch::new();

#[embassy_executor::task]
pub(super) async fn task(mut backlight: Output<'static>) {
    let mut mode_rx = BACKLIGHT_MODE.receiver().unwrap();
    let wake_rx = BACKLIGHT_WAKE.receiver();
    let on_tx = BACKLIGHT_ON.sender();

    let mut mode = BacklightMode::Auto;
    let mut backlight_off_time = None;

    // Backlight is on at boot
    BACKLIGHT_WAKE.send(()).await;

    loop {
        match select3(
            mode_rx.changed(),
            wake_rx.receive(),
            MaybeTimer::at(backlight_off_time),
        )
        .await
        {
            Either3::First(new_mode) => {
                mode = new_mode;
            }
            Either3::Second(_) => {
                backlight_off_time = Some(Instant::now() + Duration::from_secs(30));
            }
            Either3::Third(_) => {
                backlight_off_time = None;
            }
        }

        info!("Backlight mode: {}", mode);
        match mode {
            BacklightMode::AlwaysOn => {
                on_tx.send(true);
                backlight.set_high();
            }
            BacklightMode::Auto => {
                info!("Backlight off at: {}", backlight_off_time);
                if backlight_off_time.is_none() {
                    on_tx.send(false);
                    backlight.set_low();
                } else {
                    on_tx.send(true);
                    backlight.set_high();
                }
            }
        }
    }
}
