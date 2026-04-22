use crate::{StatusLightResources, network::NUM_LISTENERS};
use defmt::{Format, warn};
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Timer, with_timeout};
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) struct Request;
#[derive(Clone, Format)]
pub(crate) struct Response;

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait StatusLightInterfaceChannel {
    async fn set(&mut self) -> Result<(), ()>;
}

impl StatusLightInterfaceChannel for TheirChannelSide {
    async fn set(&mut self) -> Result<(), ()> {
        self.send(Request).await;

        match with_timeout(Duration::from_millis(200), self.receive()).await {
            Ok(_) => Ok(()),
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn task(r: StatusLightResources, comm: [MyChannelSide; NUM_LISTENERS]) {
    let mut red = Output::new(r.red, Level::Low);
    let mut amber = Output::new(r.amber, Level::Low);
    let mut green = Output::new(r.green, Level::Low);

    loop {
        red.set_high();
        amber.set_low();
        green.set_low();
        Timer::after(Duration::from_secs(1)).await;

        red.set_low();
        amber.set_high();
        green.set_low();
        Timer::after(Duration::from_secs(1)).await;

        red.set_low();
        amber.set_low();
        green.set_high();
        Timer::after(Duration::from_secs(1)).await;
    }

    loop {
        let rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());
        let (request, idx) = embassy_futures::select::select_array(rx_futures).await;

        // TODO

        comm[idx].send(Response).await;
    }
}
