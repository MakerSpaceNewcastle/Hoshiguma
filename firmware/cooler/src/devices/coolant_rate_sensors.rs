use crate::{CoolantRateSensorResources, api::NUM_LISTENERS};
use defmt::{Format, error, warn};
use embassy_executor::Spawner;
use embassy_futures::select::Either;
use embassy_rp::{
    gpio::Pull,
    pwm::{Config as PwmConfig, InputMode, Pwm},
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_time::{Duration, Ticker, with_timeout};
use hoshiguma_api::cooler::RawCoolantRate;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel = BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response>;

#[derive(Clone, Format)]
pub(crate) enum Request {
    GetRawRate,
    GetTotalPulses,
}
#[derive(Clone, Format)]
pub(crate) enum Response {
    RawRate(RawCoolantRate),
    TotalPulses(u64),
}

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

pub(crate) trait CoolantRateInterfaceChannel {
    async fn get_rate(&mut self) -> Result<RawCoolantRate, ()>;
    async fn get_total_pulses(&mut self) -> Result<u64, ()>;
}

impl CoolantRateInterfaceChannel for TheirChannelSide {
    async fn get_rate(&mut self) -> Result<RawCoolantRate, ()> {
        self.send(Request::GetRawRate).await;

        match with_timeout(Duration::from_millis(1200), self.receive()).await {
            Ok(Response::RawRate(rate)) => Ok(rate),
            Ok(_) => {
                error!("Incorrect response type");
                Err(())
            }
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }

    async fn get_total_pulses(&mut self) -> Result<u64, ()> {
        self.send(Request::GetTotalPulses).await;

        match with_timeout(Duration::from_millis(1200), self.receive()).await {
            Ok(Response::TotalPulses(rate)) => Ok(rate),
            Ok(_) => {
                error!("Incorrect response type");
                Err(())
            }
            Err(_) => {
                warn!("Timeout");
                Err(())
            }
        }
    }
}

pub(crate) fn start(
    spawner: Spawner,
    r: CoolantRateSensorResources,
    flow_comm: [MyChannelSide; NUM_LISTENERS],
    return_comm: [MyChannelSide; NUM_LISTENERS],
) {
    let flow_pwm = Pwm::new_input(
        r.flow_pwm,
        r.flow_pin,
        Pull::Down,
        InputMode::RisingEdge,
        PwmConfig::default(),
    );

    let return_pwm = Pwm::new_input(
        r.return_pwm,
        r.return_pin,
        Pull::Down,
        InputMode::RisingEdge,
        PwmConfig::default(),
    );

    spawner.spawn(task(flow_pwm, flow_comm).unwrap());
    spawner.spawn(task(return_pwm, return_comm).unwrap());
}

const MEASUREMENT_INTERVAL: Duration = Duration::from_secs(2);
const MEASUREMENT_INTERVAL_CORE: core::time::Duration = core::time::Duration::from_secs(2);

#[embassy_executor::task(pool_size = 2)]
async fn task(pwm: Pwm<'static>, comm: [MyChannelSide; NUM_LISTENERS]) {
    let mut ticker = Ticker::every(MEASUREMENT_INTERVAL);

    let mut total_pulses = 0_u64;
    let mut rate = RawCoolantRate::new(0, MEASUREMENT_INTERVAL_CORE);

    loop {
        let comm_rx_futures: [_; NUM_LISTENERS] = comm.each_ref().map(|f| f.receive());

        match embassy_futures::select::select(
            ticker.next(),
            embassy_futures::select::select_array(comm_rx_futures),
        )
        .await
        {
            // Take a measurement
            Either::First(_) => {
                // Read pulses since last sample
                let pulses = pwm.counter();
                pwm.set_counter(0);

                // Record the total number of pulses since boot
                total_pulses = total_pulses.saturating_add(pulses.into());

                rate = RawCoolantRate::new(pulses, MEASUREMENT_INTERVAL_CORE);
            }
            // Respond to a request for the measurement
            Either::Second((Request::GetRawRate, idx)) => {
                comm[idx].send(Response::RawRate(rate)).await;
            }
            // Respond to a request for the total pulses
            Either::Second((Request::GetTotalPulses, idx)) => {
                comm[idx].send(Response::TotalPulses(total_pulses)).await;
            }
        }
    }
}
