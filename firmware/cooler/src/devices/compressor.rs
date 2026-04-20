use crate::CompressorResources;
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::WaitResult};
use embassy_time::{Duration, with_timeout};
use hoshiguma_api::cooler::CompressorState;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<
    'static,
    CriticalSectionRawMutex,
    Request,
    Response,
    4,
    { crate::network::NUM_LISTENERS },
    1,
>;

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
        self.to_you.publish(Request::Set(state.clone())).await;

        match with_timeout(Duration::from_millis(200), self.to_me.next_message()).await {
            Ok(WaitResult::Message(response_state)) => {
                if response_state.0 == state {
                    Ok(response_state.0)
                } else {
                    Err(())
                }
            }
            Ok(WaitResult::Lagged(n)) => {
                panic!("Lagged by {n} messages");
            }
            Err(_) => Err(()),
        }
    }

    async fn get(&mut self) -> Result<CompressorState, ()> {
        self.to_you.publish(Request::Get).await;

        match with_timeout(Duration::from_millis(200), self.to_me.next_message()).await {
            Ok(WaitResult::Message(response_state)) => Ok(response_state.0),
            Ok(WaitResult::Lagged(n)) => {
                panic!("Lagged by {n} messages");
            }
            Err(_) => Err(()),
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn task(r: CompressorResources, mut comm: MyChannelSide) -> ! {
    let mut output = Output::new(r.relay, Level::Low);

    loop {
        match comm.to_me.next_message().await {
            WaitResult::Lagged(n) => {
                panic!("Lagged by {n} messages");
            }
            WaitResult::Message(msg) => {
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
                comm.to_you.publish(Response(state)).await;
            }
        }
    }
}
