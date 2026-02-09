#![no_std]
#![no_main]

mod air_assist;
mod coolant_rate;
mod cooling;
mod extraction_airflow;
mod fume_extraction;
mod hmi_status_screen;
mod interlock;
mod machine_power;
mod status_light;
mod temperatures;

use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, with_timeout};
use hoshiguma_state_machines::StateMachineRun;
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _ = embassy_rp::init(Default::default());

    air_assist::test_basic().await;
    coolant_rate::test_rate().await;
    coolant_rate::test_rate_symmetry_pump_start().await;
    coolant_rate::test_rate_symmetry_pump_stop().await;
    cooling::test_basic().await;
    extraction_airflow::test_basic().await;
    fume_extraction::test_basic().await;
    fume_extraction::test_mode().await;
    hmi_status_screen::test_states().await;
    hmi_status_screen::test_states_debounce().await;
    hmi_status_screen::test_statuses_to_messages().await;
    interlock::test_init_denied().await;
    interlock::test_become_happy_then_get_sad().await;
    interlock::test_lockout().await;
    interlock::test_allow_until_idle().await;
    machine_power::test_basic().await;
    status_light::test_basic().await;
    temperatures::test_basic().await;
    temperatures::test_electronics_temperature().await;
    temperatures::test_coolant_flow_temperature().await;
    temperatures::test_coolant_reservoir_temperature().await;
    temperatures::test_failed_sensor().await;

    info!("End of tests!");
}

async fn run_test<R: StateMachineRun, F: AsyncFnMut() -> ()>(
    timeout: Duration,
    mut runner: R,
    mut test_fn: F,
) {
    let name = core::any::type_name_of_val(&test_fn)
        .strip_suffix("::{{closure}}::{{closure}}")
        .unwrap();

    info!("/==\\ {}", name);

    match with_timeout(timeout, select(runner.run(), test_fn())).await {
        Ok(Either::First(_)) => {
            unreachable!()
        }
        Ok(Either::Second(_)) => {
            info!("\\==/ {} OK", name);
        }
        Err(_) => {
            panic!("Test {} timed out", name);
        }
    }
}

#[macro_export]
macro_rules! assert_duration {
    ($before: expr, $after: expr, $expected: expr, $tolerance: expr) => {{
        let duration = $after - $before;
        let lower_bound = $expected - $tolerance;
        let upper_bound = $expected + $tolerance;
        assert!(
            duration >= lower_bound && duration <= upper_bound,
            "Duration {:?} is not approximately {:?} (±{:?})",
            duration,
            $expected,
            $tolerance
        );
    }};
}

#[macro_export]
macro_rules! assert_queue_empty {
    ($comm: expr) => {{
        embassy_time::Timer::after_millis(25).await;
        assert_eq!($comm.receive_channel_len(), 0);
    }};
}
