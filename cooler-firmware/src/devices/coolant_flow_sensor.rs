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

const PULSES_PER_LITRE: f64 = 400.0;
const MEASUREMENT_INTERVAL: Duration = Duration::from_secs(2);

#[embassy_executor::task]
async fn task(r: FlowSensorResources) {
    let pwm = Pwm::new_input(
        r.pwm,
        r.pin,
        Pull::Down,
        InputMode::RisingEdge,
        PwmConfig::default(),
    );
    let mut ticker = Ticker::every(MEASUREMENT_INTERVAL);

    let mut total_pulses = 0u64;

    loop {
        ticker.next().await;

        // Read pulses since last sample
        let pulses = pwm.counter();
        pwm.set_counter(0);

        // Keep a running total of pulses for calibration purposes
        total_pulses = total_pulses.wrapping_add(pulses.into());
        info!("Total pulses since boot: {}", total_pulses);

        let litres = pulses as f64 / PULSES_PER_LITRE;
        let seconds = MEASUREMENT_INTERVAL.as_secs() as f64;
        let litres_per_minute = (litres / seconds) * 60.0;

        info!(
            "Flow: {} pulses, {} litres, in {} seconds, {} L/min",
            pulses, litres, seconds, litres_per_minute
        );

        let flow = CoolantFlow::new(litres_per_minute);

        READING.lock().await.replace(flow);
    }
}

pub(crate) struct CoolantFlowSensor {}

impl CoolantFlowSensor {
    pub(crate) fn new(spawner: &Spawner, r: FlowSensorResources) -> Self {
        unwrap!(spawner.spawn(task(r)));
        Self {}
    }

    pub(crate) async fn get(&self) -> CoolantFlow {
        READING.lock().await.borrow().clone()
    }
}
