use crate::api::NUM_LISTENERS;
use defmt::{Format, info, warn};
use embassy_futures::select::Either;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, with_timeout};
use hoshiguma_api::HmiBacklightMode;
use hoshiguma_common::{
    bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides},
    maybe_timer::MaybeTimer,
};
use peek_o_display_bsp::embassy_rp::gpio::Output;

pub(crate) const NUM_COMM_CHANNELS: usize = NUM_LISTENERS + 1;

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) enum Request {
    SetMode(HmiBacklightMode),
    Wake,
}
#[derive(Clone, Format, PartialEq, Eq)]
pub(crate) enum Response {
    Mode(HmiBacklightMode),
    Wake,
}

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait BacklightInterfaceChannel {
    async fn set_mode(&mut self, mode: HmiBacklightMode) -> Result<HmiBacklightMode, ()>;
    async fn wake(&mut self) -> Result<(), ()>;
}

impl BacklightInterfaceChannel for TheirChannelSide {
    async fn set_mode(&mut self, mode: HmiBacklightMode) -> Result<HmiBacklightMode, ()> {
        self.send(Request::SetMode(mode.clone())).await;

        match with_timeout(Duration::from_millis(200), self.receive()).await {
            Ok(response) => {
                if response == crate::devices::backlight::Response::Mode(mode.clone()) {
                    Ok(mode)
                } else {
                    warn!("Response mismatch");
                    Err(())
                }
            }
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }

    async fn wake(&mut self) -> Result<(), ()> {
        self.send(Request::Wake).await;

        match with_timeout(Duration::from_millis(200), self.receive()).await {
            Ok(response) => {
                if response == crate::devices::backlight::Response::Wake {
                    Ok(())
                } else {
                    warn!("Response mismatch");
                    Err(())
                }
            }
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn task(mut backlight: Output<'static>, comm: [MyChannelSide; NUM_COMM_CHANNELS]) {
    let mut mode = HmiBacklightMode::Auto;
    let mut backlight_off_time = Some(Instant::now() + Duration::from_secs(30));

    loop {
        let rx_futures: [_; NUM_COMM_CHANNELS] = comm.each_ref().map(|f| f.receive());
        let comm_future = embassy_futures::select::select_array(rx_futures);

        match embassy_futures::select::select(comm_future, MaybeTimer::at(backlight_off_time)).await
        {
            Either::First((req, idx)) => match req {
                Request::SetMode(new_mode) => {
                    mode = new_mode;
                    comm[idx].send(Response::Mode(mode.clone())).await;
                }
                Request::Wake => {
                    backlight_off_time = Some(Instant::now() + Duration::from_secs(30));
                    comm[idx].send(Response::Wake).await;
                }
            },
            Either::Second(_) => {
                backlight_off_time = None;
            }
        }

        info!("Backlight mode: {}", mode);
        match mode {
            HmiBacklightMode::AlwaysOn => {
                backlight.set_high();
            }
            HmiBacklightMode::Auto => {
                info!("Backlight off at: {}", backlight_off_time);
                if backlight_off_time.is_none() {
                    backlight.set_low();
                } else {
                    backlight.set_high();
                }
            }
        }
    }
}
