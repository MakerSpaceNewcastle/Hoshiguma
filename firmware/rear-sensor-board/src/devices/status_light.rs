use crate::{StatusLightResources, network::NUM_LISTENERS};
use defmt::{Format, warn};
use embassy_futures::select::Either;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Instant, Ticker, with_timeout};
use hoshiguma_api::rear_sensor_board::{LightPattern, LightState, StatusLightSettings};
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) struct Request(StatusLightSettings);
#[derive(Clone, Format)]
pub(crate) struct Response(StatusLightSettings);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait StatusLightInterfaceChannel {
    async fn set(&mut self, settings: StatusLightSettings) -> Result<StatusLightSettings, ()>;
}

impl StatusLightInterfaceChannel for TheirChannelSide {
    async fn set(&mut self, settings: StatusLightSettings) -> Result<StatusLightSettings, ()> {
        self.send(Request(settings)).await;

        match with_timeout(Duration::from_millis(200), self.receive()).await {
            Ok(settings) => Ok(settings.0),
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

    let mut settings = StatusLightSettings::default();
    let mut time_zero = Instant::now();

    let mut ticker = Ticker::every(Duration::from_millis(
        LightPattern::SEQUENCE_DURATION.as_millis() as u64,
    ));

    loop {
        let comm_rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());

        match embassy_futures::select::select(
            ticker.next(),
            embassy_futures::select::select_array(comm_rx_futures),
        )
        .await
        {
            Either::First(_) => {
                let now = core::time::Duration::from_millis(
                    Instant::now().duration_since(time_zero).as_millis() as u64,
                );

                for (output, pattern) in [
                    (&mut red, &settings.red),
                    (&mut amber, &settings.amber),
                    (&mut green, &settings.green),
                ] {
                    let state = pattern.state_at_time(now);
                    output.set_level(match state {
                        LightState::On => Level::High,
                        LightState::Off => Level::Low,
                    });
                }
            }
            Either::Second((request, idx)) => {
                settings = request.0;
                time_zero = Instant::now();
                ticker.reset();
                comm[idx].send(Response(settings.clone())).await;
            }
        }
    }
}
