use super::safety::monitor::MONITORS_CHANGED;
use crate::{
    changed::ObservedValue,
    devices::{
        cooler::{CoolerControlCommand, COOLER_CONTROL_COMMAND},
        machine_power_detector::MACHINE_POWER_CHANGED,
    },
    telemetry::queue_telemetry_event,
};
use defmt::unwrap;
use embassy_futures::select::{select, Either};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::Publisher, watch::Watch};
use hoshiguma_protocol::{
    cooler::types::{Compressor, CoolantPump, RadiatorFan, Stirrer},
    peripheral_controller::{
        event::EventKind,
        types::{CoolingDemand, CoolingEnabled, MachinePower, MonitorKind},
    },
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn power_control() {
    let mut machine_power_rx = MACHINE_POWER_CHANGED.receiver().unwrap();
    let mut monitor_rx = MONITORS_CHANGED.receiver().unwrap();

    let cooler_command_tx = unwrap!(COOLER_CONTROL_COMMAND.publisher());

    let mut enabled = ObservedValue::new(CoolingEnabled::Inhibit);
    let mut comms_is_ok = false;

    loop {
        match select(machine_power_rx.changed(), monitor_rx.changed()).await {
            Either::First(power) => {
                enabled
                    .update_and_async(
                        match power {
                            MachinePower::On => CoolingEnabled::Enable,
                            MachinePower::Off => CoolingEnabled::Inhibit,
                        },
                        |enabled| async {
                            queue_telemetry_event(EventKind::CoolingEnableChanged(enabled.clone()))
                                .await;
                            send_cooler_command(enabled, &cooler_command_tx).await;
                        },
                    )
                    .await;
            }
            Either::Second(monitors) => {
                let comms_is_ok_now =
                    *monitors.get(MonitorKind::CoolerCommunicationFault) == Severity::Normal;
                if !comms_is_ok && comms_is_ok_now {
                    send_cooler_command(enabled.clone(), &cooler_command_tx).await;
                    comms_is_ok = true;
                }
            }
        }
    }
}

async fn send_cooler_command<const CAP: usize, const SUBS: usize, const PUBS: usize>(
    enabled: CoolingEnabled,
    tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, CAP, SUBS, PUBS>,
) {
    tx.publish(CoolerControlCommand::SetCoolantPump(match enabled {
        CoolingEnabled::Inhibit => CoolantPump::Idle,
        CoolingEnabled::Enable => CoolantPump::Run,
    }))
    .await;

    // TODO: delay?

    tx.publish(CoolerControlCommand::SetStirrer(match enabled {
        CoolingEnabled::Inhibit => Stirrer::Idle,
        CoolingEnabled::Enable => Stirrer::Run,
    }))
    .await;
}

pub(crate) static COOLING_DEMAND: Watch<CriticalSectionRawMutex, CoolingDemand, 1> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn cooling_control() {
    let mut cooling_demand_rx = COOLING_DEMAND.receiver().unwrap();

    let cooler_command_tx = unwrap!(COOLER_CONTROL_COMMAND.publisher());

    // TODO: validation of cooler state

    loop {
        let demand = cooling_demand_rx.changed().await;
        queue_telemetry_event(EventKind::CoolingDemandChanged(demand.clone())).await;

        match demand {
            CoolingDemand::Demand => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetRadiatorFan(RadiatorFan::Run))
                    .await;
                // TODO: delay
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCompressor(Compressor::Run))
                    .await;
            }
            CoolingDemand::Idle => {
                cooler_command_tx
                    .publish(CoolerControlCommand::SetRadiatorFan(RadiatorFan::Idle))
                    .await;
                // TODO: delay
                cooler_command_tx
                    .publish(CoolerControlCommand::SetCompressor(Compressor::Idle))
                    .await;
            }
        }
    }
}

#[embassy_executor::task]
pub(crate) async fn thermal_monitor() {
    loop {
        // TODO
        embassy_time::Timer::after_secs(10).await;
    }
}
