use crate::CoolantPumpResources;
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::WaitResult};
use hoshiguma_api::cooler::CoolantPumpState;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel =
    BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response, 4, 1, 1>;

#[derive(Clone, Format)]
pub(crate) struct Request(CoolantPumpState);
#[derive(Clone, Format)]
pub(crate) struct Response(CoolantPumpState);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

#[embassy_executor::task]
pub(crate) async fn task(r: CoolantPumpResources, mut comm: MyChannelSide) -> ! {
    let mut output = Output::new(r.relay, Level::Low);

    loop {
        match comm.to_me.next_message().await {
            WaitResult::Lagged(n) => {
                panic!("Lagged by {n} messages");
            }
            WaitResult::Message(msg) => {
                output.set_level(match msg.0 {
                    CoolantPumpState::Idle => Level::Low,
                    CoolantPumpState::Run => Level::High,
                });

                comm.to_you.publish(Response(msg.0)).await;
            }
        }
    }
}
