use crate::{CompressorResources, api::NUM_LISTENERS};
use defmt::{Format, warn};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, with_timeout};
use hoshiguma_api::cooler::CompressorState;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) enum Request {
    Get,
    Set(CompressorState),
}
#[derive(Clone, Format)]
pub(crate) struct Response(CompressorState);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait CompressorInterfaceChannel {
    async fn set(&mut self, state: CompressorState) -> Result<CompressorState, ()>;
    async fn get(&mut self) -> Result<CompressorState, ()>;
}

impl CompressorInterfaceChannel for TheirChannelSide {
    async fn set(&mut self, state: CompressorState) -> Result<CompressorState, ()> {
        self.send(Request::Set(state.clone())).await;

        if self.get().await? == state {
            Ok(state)
        } else {
            warn!("Response mismatch");
            Err(())
        }
    }

    async fn get(&mut self) -> Result<CompressorState, ()> {
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
pub(crate) async fn task(r: CompressorResources, comm: [MyChannelSide; NUM_LISTENERS]) -> ! {
    let mut output = Output::new(r.relay, Level::Low);

    loop {
        let rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());
        let (msg, idx) = embassy_futures::select::select_array(rx_futures).await;

        if let Request::Set(state) = msg {
            output.set_level(match state {
                CompressorState::Idle => Level::Low,
                CompressorState::Run => Level::High,
            });
        }

        let state = match output.get_output_level() {
            Level::Low => CompressorState::Idle,
            Level::High => CompressorState::Run,
        };
        comm[idx].send(Response(state)).await;
    }
}
