use crate::{CoolantPumpResources, api::NUM_LISTENERS};
use defmt::{Format, warn};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, with_timeout};
use hoshiguma_api::cooler::CoolantPumpState;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) enum Request {
    Get,
    Set(CoolantPumpState),
}
#[derive(Clone, Format)]
pub(crate) struct Response(CoolantPumpState);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait CoolantPumpInterfaceChannel {
    async fn set(&mut self, state: CoolantPumpState) -> Result<CoolantPumpState, ()>;
    async fn get(&mut self) -> Result<CoolantPumpState, ()>;
}

impl CoolantPumpInterfaceChannel for TheirChannelSide {
    async fn set(&mut self, state: CoolantPumpState) -> Result<CoolantPumpState, ()> {
        self.send(Request::Set(state.clone())).await;

        if self.get().await? == state {
            Ok(state)
        } else {
            warn!("Response mismatch");
            Err(())
        }
    }

    async fn get(&mut self) -> Result<CoolantPumpState, ()> {
        self.send(Request::Get).await;

        match with_timeout(Duration::from_millis(200), self.receive()).await {
            Ok(response) => Ok(response.0),
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn task(r: CoolantPumpResources, comm: [MyChannelSide; NUM_LISTENERS]) -> ! {
    let mut output = Output::new(r.relay, Level::Low);

    loop {
        let rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());
        let (msg, idx) = embassy_futures::select::select_array(rx_futures).await;

        if let Request::Set(state) = msg {
            output.set_level(match state {
                CoolantPumpState::Idle => Level::Low,
                CoolantPumpState::Run => Level::High,
            });
        }

        let state = match output.get_output_level() {
            Level::Low => CoolantPumpState::Idle,
            Level::High => CoolantPumpState::Run,
        };
        comm[idx].send(Response(state)).await;
    }
}
