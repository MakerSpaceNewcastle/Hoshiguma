use crate::{
    changed::ObservedValue, devices::cooler::COOLER_TEMPERATURES_READ,
    logic::cooling::control::COOLING_DEMAND,
};
use defmt::info;
use embassy_time::{Duration, Instant};
use hoshiguma_protocol::peripheral_controller::types::CoolingDemand;

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cooler demand logic").await;

    let mut temperatures_rx = COOLER_TEMPERATURES_READ.receiver().unwrap();

    let cooling_demand_tx = COOLING_DEMAND.sender();

    let mut demand = ObservedValue::new(CoolingDemand::Idle);
    let mut last_state_change = None;

    const UPPER_TEMPERATURE: f32 = 17.5;
    const LOWER_TEMPERATURE: f32 = 17.0;

    loop {
        let temperatures = temperatures_rx.changed().await;

        if let Ok(res_flow_loop_temperature) = temperatures.reservoir_left_side {
            // Get the new demand based on temperature alone, with hysteresis
            let new_demand = if res_flow_loop_temperature > UPPER_TEMPERATURE {
                Some(CoolingDemand::Demand)
            } else if res_flow_loop_temperature < LOWER_TEMPERATURE {
                Some(CoolingDemand::Idle)
            } else {
                None
            };
            info!(
                "Reservoir flow temperature {} calls for cooling demand {}",
                res_flow_loop_temperature, new_demand
            );

            // Determine if the state should be allowed to change now based on elapsed time since
            // the last commanded state change
            let now = Instant::now();
            let state_change_allowed = match last_state_change {
                Some(time) => {
                    // Send a demand change command at most every 60 seconds to avoid cycling the compressor
                    // too often
                    now.saturating_duration_since(time) >= Duration::from_secs(60)
                }
                None => {
                    // If no previous command was sent then send the demand command immediately
                    true
                }
            };
            info!(
                "Demand last changed at {}, now {}, change allowed: {}",
                last_state_change, now, state_change_allowed
            );

            // If the final demand command is new, then send it
            if let Some(new_demand) = new_demand {
                if state_change_allowed {
                    demand.update_and(new_demand, |demand| {
                        info!("Cooling demand: {}", demand);
                        cooling_demand_tx.send(demand);
                        last_state_change = Some(now);
                    });
                }
            }
        }
    }
}
