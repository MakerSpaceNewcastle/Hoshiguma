use crate::{CompressorResources, network::NUM_LISTENERS};
use defmt::{Format, warn};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, with_timeout};
use hoshiguma_api::cooler::CompressorState;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel =
    BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response, 4>;

#[derive(Clone, Format)]
pub(crate) enum Request {
    Get,
    Set(CompressorState),
}
#[derive(Clone, Format)]
pub(crate) struct Response(CompressorState);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait CompressorControlChannel {
    async fn set(&mut self, state: CompressorState) -> Result<CompressorState, ()>;
    async fn get(&mut self) -> Result<CompressorState, ()>;
}

impl CompressorControlChannel for TheirChannelSide {
    async fn set(&mut self, state: CompressorState) -> Result<CompressorState, ()> {
        self.to_you.send(Request::Set(state.clone())).await;

        match with_timeout(Duration::from_millis(200), self.to_me.receive()).await {
            Ok(response_state) => {
                if response_state.0 == state {
                    Ok(response_state.0)
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

    async fn get(&mut self) -> Result<CompressorState, ()> {
        self.to_you.send(Request::Get).await;

        match with_timeout(Duration::from_millis(200), self.to_me.receive()).await {
            Ok(response_state) => Ok(response_state.0),
            Err(_) => Err(()),
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn task(r: CompressorResources, comm: [MyChannelSide; NUM_LISTENERS]) -> ! {
    let mut output = Output::new(r.relay, Level::Low);

    loop {
        let rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.to_me.receive());
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
        comm[idx].to_you.send(Response(state)).await;
    }
}
