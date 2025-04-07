use super::safety::monitor::MONITORS_CHANGED;
use crate::{
    changed::ObservedValue,
    devices::{
        cooler::{CoolerControlCommand, COOLER_CONTROL_COMMAND},
        machine_power_detector::MACHINE_POWER_CHANGED,
    },
    telemetry::queue_telemetry_event,
};
use defmt::{info, unwrap};
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
                            send_cooler_enable_command(enabled, &cooler_command_tx).await;
                        },
                    )
                    .await;
            }
            Either::Second(monitors) => {
                let comms_is_ok_now =
                    *monitors.get(MonitorKind::CoolerCommunicationFault) == Severity::Normal;
                if !comms_is_ok && comms_is_ok_now {
                    info!("Communications restored, resending cooler enable command");
                    send_cooler_enable_command(enabled.clone(), &cooler_command_tx).await;
                }
                comms_is_ok = comms_is_ok_now;
            }
        }
    }
}

async fn send_cooler_enable_command<const CAP: usize, const SUBS: usize, const PUBS: usize>(
    enabled: CoolingEnabled,
    tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, CAP, SUBS, PUBS>,
) {
    tx.publish(CoolerControlCommand::SetCoolantPump(match enabled {
        CoolingEnabled::Inhibit => CoolantPump::Idle,
        CoolingEnabled::Enable => CoolantPump::Run,
    }))
    .await;

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

    // TODO: validation of cooler state (as per above)

    loop {
        let demand = cooling_demand_rx.changed().await;
        queue_telemetry_event(EventKind::CoolingDemandChanged(demand.clone())).await;
        send_cooler_demand_command(demand, &cooler_command_tx).await;
    }
}

async fn send_cooler_demand_command<const CAP: usize, const SUBS: usize, const PUBS: usize>(
    demand: CoolingDemand,
    tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, CAP, SUBS, PUBS>,
) {
    tx.publish(CoolerControlCommand::SetRadiatorFan(match demand {
        CoolingDemand::Idle => RadiatorFan::Idle,
        CoolingDemand::Demand => RadiatorFan::Run,
    }))
    .await;

    tx.publish(CoolerControlCommand::SetCompressor(match demand {
        CoolingDemand::Idle => Compressor::Idle,
        CoolingDemand::Demand => Compressor::Run,
    }))
    .await;
}

#[embassy_executor::task]
pub(crate) async fn thermal_monitor() {
    let cooling_demand_tx = COOLING_DEMAND.sender();

    loop {
        // TODO
        cooling_demand_tx.send(CoolingDemand::Idle);
        embassy_time::Timer::after_secs(30).await;
        cooling_demand_tx.send(CoolingDemand::Demand);
        embassy_time::Timer::after_secs(30).await;
    }
}
