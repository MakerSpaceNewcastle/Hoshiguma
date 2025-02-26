use crate::{
    devices::{
        air_assist::{AirAssistDemand, AIR_ASSIST_DEMAND_CHANGED, AIR_ASSIST_PUMP},
        machine_power_detector::{MachinePower, MACHINE_POWER_CHANGED},
    },
    maybe_timer::MaybeTimer,
};
use defmt::{debug, info, unwrap, Format};
use embassy_time::{Duration, Instant};

#[derive(Clone, Format)]
enum AirAssistState {
    Idle,
    RunOn(Instant),
    Demand,
}

impl From<&AirAssistState> for AirAssistDemand {
    fn from(state: &AirAssistState) -> Self {
        match state {
            AirAssistState::Idle => Self::Idle,
            AirAssistState::RunOn(_) => Self::Demand,
            AirAssistState::Demand => Self::Demand,
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(1);

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut state = AirAssistState::Idle;

    let mut machine_power_rx = unwrap!(MACHINE_POWER_CHANGED.receiver());
    let mut demand_rx = unwrap!(AIR_ASSIST_DEMAND_CHANGED.receiver());
    let pump_tx = AIR_ASSIST_PUMP.sender();

    let mut machine_power = MachinePower::Off;

    loop {
        let run_on_timer = MaybeTimer::at(if let AirAssistState::RunOn(t) = state {
            Some(t)
        } else {
            None
        });

        let new_state = {
            use embassy_futures::select::{select3, Either3};

            match select3(
                machine_power_rx.changed(),
                demand_rx.changed(),
                run_on_timer,
            )
            .await
            {
                Either3::First(power) => {
                    machine_power = power;
                    state.clone()
                }
                Either3::Second(demand) => match demand {
                    AirAssistDemand::Idle => match state {
                        AirAssistState::Idle => AirAssistState::Idle,
                        AirAssistState::RunOn(t) => AirAssistState::RunOn(t),
                        AirAssistState::Demand => AirAssistState::RunOn(Instant::now() + TIMEOUT),
                    },
                    AirAssistDemand::Demand => AirAssistState::Demand,
                },
                Either3::Third(()) => {
                    debug!("Run on timer expired");
                    AirAssistState::Idle
                }
            }
        };

        info!("Air assist state {} -> {}", state, new_state);

        // Turn off demand relay if the 24V bus is not powered.
        pump_tx.send(match machine_power {
            MachinePower::Off => AirAssistDemand::Idle,
            MachinePower::On => (&new_state).into(),
        });

        state = new_state;
    }
}
