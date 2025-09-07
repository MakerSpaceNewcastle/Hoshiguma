use crate::{
    changed::ObservedValue,
    devices::{
        cooler::{CoolerControlCommand, COOLER_CONTROL_COMMAND, COOLER_TEMPERATURES_READ},
        machine_power_detector::MACHINE_POWER_CHANGED,
    },
    logic::safety::monitor::MONITORS_CHANGED,
    telemetry::queue_telemetry_event,
};
use defmt::{info, unwrap};
use embassy_futures::select::{select4, Either4};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::Publisher, watch::Watch};
use hoshiguma_protocol::{
    cooler::types::{CompressorState, CoolantPumpState, RadiatorFanState, Temperatures},
    peripheral_controller::{
        event::EventKind,
        types::{CoolingDemand, CoolingEnabled, MachinePower, MonitorKind},
    },
    types::Severity,
};

pub(crate) static COOLING_DEMAND: Watch<CriticalSectionRawMutex, CoolingDemand, 1> = Watch::new();

/// Temperature thresholds for radiator fan operation due to electronics heating
const ELECTRONICS_TEMPERATURE_THRESHOLD: f32 = 30.0; // °C for onboard and ambient sensors
const PUMP_MOTOR_TEMPERATURE_THRESHOLD: f32 = 60.0; // °C for coolant pump motor

/// Check if the cooler electronics are hot and require radiator fan for cooling
fn are_electronics_hot(temperatures: &Temperatures) -> bool {
    // Check onboard temperature sensor
    if let Ok(temp) = temperatures.onboard {
        if temp > ELECTRONICS_TEMPERATURE_THRESHOLD {
            return true;
        }
    }

    // Check internal ambient temperature
    if let Ok(temp) = temperatures.internal_ambient {
        if temp > ELECTRONICS_TEMPERATURE_THRESHOLD {
            return true;
        }
    }

    // Check coolant pump motor temperature
    if let Ok(temp) = temperatures.coolant_pump_motor {
        if temp > PUMP_MOTOR_TEMPERATURE_THRESHOLD {
            return true;
        }
    }

    false
}

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cooler control logic").await;

    let mut machine_power_rx = MACHINE_POWER_CHANGED.receiver().unwrap();
    let mut cooling_demand_rx = COOLING_DEMAND.receiver().unwrap();
    let mut monitor_rx = MONITORS_CHANGED.receiver().unwrap();
    let mut temperatures_rx = COOLER_TEMPERATURES_READ.receiver().unwrap();

    let cooler_command_tx = unwrap!(COOLER_CONTROL_COMMAND.publisher());

    let mut machine_power = MachinePower::Off;
    let mut external_demand = CoolingDemand::Idle;
    let mut comms_is_ok = false;
    let mut temperatures = Temperatures {
        onboard: Err(()),
        internal_ambient: Err(()),
        reservoir_evaporator_coil: Err(()),
        reservoir_left_side: Err(()),
        reservoir_right_side: Err(()),
        coolant_pump_motor: Err(()),
    };

    let mut enabled = ObservedValue::new(CoolingEnabled::Inhibit);
    let mut demand = ObservedValue::new(CoolingDemand::Idle);

    loop {
        match select4(
            machine_power_rx.changed(),
            cooling_demand_rx.changed(),
            monitor_rx.changed(),
            temperatures_rx.changed(),
        )
        .await
        {
            Either4::First(power) => {
                machine_power = power.clone();

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

                set_demand(
                    &cooler_command_tx,
                    &mut demand,
                    &machine_power,
                    &external_demand,
                )
                .await;

                // Update radiator fan based on new machine power state and current temperatures
                send_radiator_fan_command(
                    &machine_power,
                    &external_demand,
                    &temperatures,
                    &cooler_command_tx,
                )
                .await;
            }
            Either4::Second(new_demand) => {
                external_demand = new_demand;

                set_demand(
                    &cooler_command_tx,
                    &mut demand,
                    &machine_power,
                    &external_demand,
                )
                .await;

                // Update radiator fan based on new cooling demand and current temperatures
                send_radiator_fan_command(
                    &machine_power,
                    &external_demand,
                    &temperatures,
                    &cooler_command_tx,
                )
                .await;
            }
            Either4::Third(monitors) => {
                let comms_is_ok_now =
                    *monitors.get(MonitorKind::CoolerCommunicationFault) == Severity::Normal;
                if !comms_is_ok && comms_is_ok_now {
                    info!("Communications restored, resending cooler commands");
                    send_cooler_enable_command(enabled.clone().unwrap(), &cooler_command_tx).await;
                    send_cooler_demand_command(demand.clone().unwrap(), &cooler_command_tx).await;
                    // Resend radiator fan command based on current state
                    send_radiator_fan_command(
                        &machine_power,
                        &external_demand,
                        &temperatures,
                        &cooler_command_tx,
                    )
                    .await;
                }
                comms_is_ok = comms_is_ok_now;
            }
            Either4::Fourth(new_temperatures) => {
                temperatures = new_temperatures;

                // Update radiator fan based on new temperature readings
                send_radiator_fan_command(
                    &machine_power,
                    &external_demand,
                    &temperatures,
                    &cooler_command_tx,
                )
                .await;
            }
        }
    }
}

async fn set_demand<const A: usize, const B: usize, const C: usize>(
    cooler_command_tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, A, B, C>,
    demand: &mut ObservedValue<CoolingDemand>,
    machine_power: &MachinePower,
    external_demand: &CoolingDemand,
) {
    let new_demand = match machine_power {
        MachinePower::On => external_demand,
        MachinePower::Off => &CoolingDemand::Idle,
    };

    demand
        .update_and_async(new_demand.clone(), |demand| async {
            queue_telemetry_event(EventKind::CoolingDemandChanged(demand.clone())).await;
            send_cooler_demand_command(demand, cooler_command_tx).await;
        })
        .await;
}

async fn send_cooler_enable_command<const CAP: usize, const SUBS: usize, const PUBS: usize>(
    enabled: CoolingEnabled,
    tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, CAP, SUBS, PUBS>,
) {
    tx.publish(CoolerControlCommand::CoolantPump(match enabled {
        CoolingEnabled::Inhibit => CoolantPumpState::Idle,
        CoolingEnabled::Enable => CoolantPumpState::Run,
    }))
    .await;
}

async fn send_cooler_demand_command<const CAP: usize, const SUBS: usize, const PUBS: usize>(
    demand: CoolingDemand,
    tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, CAP, SUBS, PUBS>,
) {
    tx.publish(CoolerControlCommand::Compressor(match demand {
        CoolingDemand::Idle => CompressorState::Idle,
        CoolingDemand::Demand => CompressorState::Run,
    }))
    .await;
}

/// Send radiator fan command based on cooling demand and electronics temperature
///
/// The radiator fan should run when either:
/// 1. There is a demand for cooling, OR
/// 2. The cooler electronics are hot (temperature thresholds exceeded)
async fn send_radiator_fan_command<const CAP: usize, const SUBS: usize, const PUBS: usize>(
    machine_power: &MachinePower,
    cooling_demand: &CoolingDemand,
    temperatures: &Temperatures,
    tx: &Publisher<'_, CriticalSectionRawMutex, CoolerControlCommand, CAP, SUBS, PUBS>,
) {
    let should_run = match machine_power {
        MachinePower::Off => {
            // When machine is off, only run fan if electronics are hot
            are_electronics_hot(temperatures)
        }
        MachinePower::On => {
            // When machine is on, run fan if there's cooling demand OR electronics are hot
            *cooling_demand == CoolingDemand::Demand || are_electronics_hot(temperatures)
        }
    };

    let fan_state = if should_run {
        RadiatorFanState::Run
    } else {
        RadiatorFanState::Idle
    };

    tx.publish(CoolerControlCommand::RadiatorFan(fan_state))
        .await;
}
