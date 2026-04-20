use crate::CoolantRateSensorResources;
use defmt::{Format, info};
use embassy_executor::Spawner;
use embassy_futures::select::Either;
use embassy_rp::{
    gpio::Pull,
    pwm::{Config as PwmConfig, InputMode, Pwm},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::WaitResult};
use embassy_time::{Duration, Ticker};
use hoshiguma_api::cooler::CoolantRate;
use hoshiguma_common::bidir_channel::{BiDirectionalChannel, BiDirectionalChannelSides};

pub(crate) type Channel =
    BiDirectionalChannel<'static, CriticalSectionRawMutex, Request, Response, 4, 1, 1>;

#[derive(Clone, Format)]
pub(crate) struct Request;
#[derive(Clone, Format)]
pub(crate) struct Response(CoolantRate);

pub(crate) type TheirChannelSide = <Channel as BiDirectionalChannelSides>::SideA;
pub(crate) type MyChannelSide = <Channel as BiDirectionalChannelSides>::SideB;

const PULSES_PER_LITRE_FLOW: f64 = 400.0;
const PULSES_PER_LITRE_RETURN: f64 = 400.0; // TODO: calibrate

pub(crate) fn start(
    spawner: Spawner,
    r: CoolantRateSensorResources,
    flow_comm: MyChannelSide,
    return_comm: MyChannelSide,
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

    spawner.spawn(task(flow_pwm, flow_comm, PULSES_PER_LITRE_FLOW).unwrap());
    spawner.spawn(task(return_pwm, return_comm, PULSES_PER_LITRE_RETURN).unwrap());
}

const MEASUREMENT_INTERVAL: Duration = Duration::from_secs(2);

#[embassy_executor::task(pool_size = 2)]
async fn task(pwm: Pwm<'static>, mut comm: MyChannelSide, pulses_per_litre: f64) {
    let mut ticker = Ticker::every(MEASUREMENT_INTERVAL);

    let mut total_pulses = 0u64;
    let mut rate = CoolantRate::ZERO;

    loop {
        match embassy_futures::select::select(ticker.next(), comm.to_me.next_message()).await {
            // Take a measurement
            Either::First(_) => {
                // Read pulses since last sample
                let pulses = pwm.counter();
                pwm.set_counter(0);

                // Keep a running total of pulses for calibration purposes
                total_pulses = total_pulses.wrapping_add(pulses.into());
                info!("Total pulses since boot: {}", total_pulses);

                let litres = pulses as f64 / pulses_per_litre;
                let seconds = MEASUREMENT_INTERVAL.as_secs() as f64;
                let litres_per_minute = (litres / seconds) * 60.0;

                info!(
                    "Flow: {} pulses, {} litres, in {} seconds, {} L/min",
                    pulses, litres, seconds, litres_per_minute
                );
                rate = CoolantRate::new(litres_per_minute);
            }
            Either::Second(WaitResult::Lagged(n)) => {
                panic!("Lagged by {n} messages");
            }
            // Respond to a request for the measurement
            Either::Second(WaitResult::Message(_)) => {
                comm.to_you.publish(Response(rate.clone())).await;
            }
        }
    }
}
