use crate::{
    changed::ObservedValue, devices::cooler::COOLER_TEMPERATURES_READ,
    logic::cooling::control::ELECTRONICS_COOLING_DEMAND,
};
use defmt::info;
use embassy_time::{Duration, Instant};
use hoshiguma_protocol::peripheral_controller::types::CoolingDemand;

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("electronics cooling demand logic").await;

    let mut temperatures_rx = COOLER_TEMPERATURES_READ.receiver().unwrap();

    let electronics_demand_tx = ELECTRONICS_COOLING_DEMAND.sender();

    let mut demand = ObservedValue::new(CoolingDemand::Idle);
    let mut last_state_change = None;

    /// Temperature thresholds for radiator fan operation due to electronics heating
    const ELECTRONICS_TEMPERATURE_THRESHOLD: f32 = 30.0; // °C for onboard and ambient sensors
    const PUMP_MOTOR_TEMPERATURE_THRESHOLD: f32 = 60.0; // °C for coolant pump motor

    loop {
        let temperatures = temperatures_rx.changed().await;

        // Check if electronics are hot
        let electronics_hot = {
            let mut hot = false;

            // Check onboard temperature sensor
            if let Ok(temp) = temperatures.onboard {
                if temp > ELECTRONICS_TEMPERATURE_THRESHOLD {
                    hot = true;
                }
            }

            // Check internal ambient temperature
            if let Ok(temp) = temperatures.internal_ambient {
                if temp > ELECTRONICS_TEMPERATURE_THRESHOLD {
                    hot = true;
                }
            }

            // Check coolant pump motor temperature
            if let Ok(temp) = temperatures.coolant_pump_motor {
                if temp > PUMP_MOTOR_TEMPERATURE_THRESHOLD {
                    hot = true;
                }
            }

            hot
        };

        // Get the new demand based on electronics temperature, with hysteresis
        let new_demand = if electronics_hot {
            Some(CoolingDemand::Demand)
        } else {
            // Use hysteresis - once electronics cool down by 2°C, turn off
            let electronics_cool = {
                let mut all_cool = true;

                if let Ok(temp) = temperatures.onboard {
                    if temp > ELECTRONICS_TEMPERATURE_THRESHOLD - 2.0 {
                        all_cool = false;
                    }
                }

                if let Ok(temp) = temperatures.internal_ambient {
                    if temp > ELECTRONICS_TEMPERATURE_THRESHOLD - 2.0 {
                        all_cool = false;
                    }
                }

                if let Ok(temp) = temperatures.coolant_pump_motor {
                    if temp > PUMP_MOTOR_TEMPERATURE_THRESHOLD - 2.0 {
                        all_cool = false;
                    }
                }

                all_cool
            };

            if electronics_cool {
                Some(CoolingDemand::Idle)
            } else {
                None // Stay in current state
            }
        };

        info!(
            "Electronics temperatures call for cooling demand: {:?}",
            new_demand
        );

        // Determine if the state should be allowed to change now based on elapsed time since
        // the last commanded state change
        let now = Instant::now();
        let state_change_allowed = match last_state_change {
            Some(time) => {
                // Send a demand change command at most every 30 seconds to avoid cycling the fan
                // too often (shorter than cooling demand since electronics protection is more urgent)
                now.saturating_duration_since(time) >= Duration::from_secs(30)
            }
            None => {
                // If no previous command was sent then send the demand command immediately
                true
            }
        };

        info!(
            "Electronics demand last changed at {:?}, now {}, change allowed: {}",
            last_state_change, now, state_change_allowed
        );

        // If the final demand command is new, then send it
        if let Some(new_demand) = new_demand {
            if state_change_allowed {
                demand.update_and(new_demand, |demand| {
                    info!("Electronics cooling demand: {}", demand);
                    electronics_demand_tx.send(demand);
                    last_state_change = Some(now);
                });
            }
        }
    }
}
