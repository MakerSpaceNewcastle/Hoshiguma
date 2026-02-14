use crate::{
    MachineRunDetectResources, polled_input::PolledInput, telemetry::queue_telemetry_data_point,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_core::{telemetry::AsTelemetry, types::MachineRun};
use pico_plc_bsp::embassy_rp::gpio::{Input, Level, Pull};

pub(crate) static MACHINE_RUNNING_CHANGED: Watch<CriticalSectionRawMutex, MachineRun, 4> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: MachineRunDetectResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("mach run det").await;

    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = MACHINE_RUNNING_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => MachineRun::Idle,
            Level::High => MachineRun::Running,
        };

        for dp in state.telemetry() {
            queue_telemetry_data_point(dp);
        }

        tx.send(state);
    }
}
