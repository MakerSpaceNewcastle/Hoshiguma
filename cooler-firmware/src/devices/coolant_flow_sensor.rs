use crate::{rpc::report_event, FlowSensorResources};
use defmt::info;
use embassy_rp::{
    gpio::Pull,
    pwm::{Config as PwmConfig, InputMode, Pwm},
};
use embassy_time::{Duration, Ticker};
use hoshiguma_protocol::cooler::{
    event::{EventKind, ObservationEvent},
    types::CoolantFlow,
};

const LITRES_PER_PULSE: f64 = 0.001; // TODO
const MEASUREMENT_INTERVAL: Duration = Duration::from_secs(2);

#[embassy_executor::task]
pub(crate) async fn task(r: FlowSensorResources) {
    let pwm = Pwm::new_input(
        r.pwm,
        r.pin,
        Pull::Down,
        InputMode::RisingEdge,
        PwmConfig::default(),
    );

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

        report_event(EventKind::Observation(ObservationEvent::CoolantFlow(flow))).await;
    }
}
