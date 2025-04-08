use crate::FlowSensorResources;
use core::cell::RefCell;
use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::Pull,
    pwm::{Config as PwmConfig, InputMode, Pwm},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Ticker};
use hoshiguma_protocol::cooler::types::CoolantFlow;

static READING: Mutex<CriticalSectionRawMutex, RefCell<CoolantFlow>> =
    Mutex::new(RefCell::new(CoolantFlow::ZERO));

const LITRES_PER_PULSE: f64 = 0.001; // TODO
const MEASUREMENT_INTERVAL: Duration = Duration::from_secs(2);

#[embassy_executor::task]
pub(crate) async fn task(pwm: Pwm<'static>) {
    let mut ticker = Ticker::every(MEASUREMENT_INTERVAL);

    loop {
        ticker.next().await;

        let pulses = pwm.counter();
        pwm.set_counter(0);

        let litres = pulses as f64 * LITRES_PER_PULSE;
        let seconds = MEASUREMENT_INTERVAL.as_secs() as f64;

        info!(
            "Flow: {} pulses, {} litres, in {} seconds",
            pulses, litres, seconds
        );

        let flow = CoolantFlow::new(litres, seconds);

        READING.lock().await.replace(flow.clone());
    }
}

pub(crate) struct CoolantFlowSensor {}

impl CoolantFlowSensor {
    pub(crate) fn new(spawner: &Spawner, r: FlowSensorResources) -> Self {
        let pwm = Pwm::new_input(
            r.pwm,
            r.pin,
            Pull::Down,
            InputMode::RisingEdge,
            PwmConfig::default(),
        );

        unwrap!(spawner.spawn(task(pwm)));

        Self {}
    }

    pub(crate) async fn get(&self) -> CoolantFlow {
        READING.lock().await.borrow().clone()
    }
}
