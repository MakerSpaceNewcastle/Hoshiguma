use crate::{
    devices::{
        fume_extraction_fan::FUME_EXTRACTION_FAN,
        fume_extraction_mode_switch::FUME_EXTRACTION_MODE_CHANGED,
        machine_power_detector::MACHINE_POWER_CHANGED,
        machine_run_detector::MACHINE_RUNNING_CHANGED,
    },
    maybe_timer::MaybeTimer,
};
use defmt::{debug, info, unwrap, Format};
use embassy_time::{Duration, Instant};
use hoshiguma_protocol::payload::{
    control::FumeExtractionFan,
    observation::{FumeExtractionMode, MachinePower, MachineRun},
};

#[derive(Clone, Format)]
enum FumeExtractionAutomaticState {
    Idle,
    RunOn(Instant),
    Demand,
}

impl From<&FumeExtractionAutomaticState> for FumeExtractionFan {
    fn from(state: &FumeExtractionAutomaticState) -> Self {
        match state {
            FumeExtractionAutomaticState::Idle => Self::Idle,
            FumeExtractionAutomaticState::RunOn(_) => Self::Run,
            FumeExtractionAutomaticState::Demand => Self::Run,
        }
    }
}

#[derive(Clone, Format)]
struct FumeExtractionState {
    mode: FumeExtractionMode,
    auto: FumeExtractionAutomaticState,
}

impl Default for FumeExtractionState {
    fn default() -> Self {
        Self {
            mode: FumeExtractionMode::Automatic,
            auto: FumeExtractionAutomaticState::Idle,
        }
    }
}

impl From<&FumeExtractionState> for FumeExtractionFan {
    fn from(state: &FumeExtractionState) -> Self {
        match state.mode {
            FumeExtractionMode::Automatic => (&state.auto).into(),
            FumeExtractionMode::OverrideRun => Self::Run,
        }
    }
}

const TIMEOUT: Duration = Duration::from_secs(45);

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut state = FumeExtractionState::default();

    let mut machine_power_rx = unwrap!(MACHINE_POWER_CHANGED.receiver());
    let mut machine_run_rx = unwrap!(MACHINE_RUNNING_CHANGED.receiver());
    let mut mode_rx = unwrap!(FUME_EXTRACTION_MODE_CHANGED.receiver());
    let fan_tx = FUME_EXTRACTION_FAN.sender();

    let mut machine_power = MachinePower::Off;

    loop {
        let run_on_timer =
            MaybeTimer::at(if let FumeExtractionAutomaticState::RunOn(t) = state.auto {
                Some(t)
            } else {
                None
            });

        let new_state = {
            use embassy_futures::select::{select4, Either4};

            match select4(
                machine_power_rx.changed(),
                machine_run_rx.changed(),
                mode_rx.changed(),
                run_on_timer,
            )
            .await
            {
                Either4::First(power) => {
                    machine_power = power;
                    state.clone()
                }
                Either4::Second(run_state) => FumeExtractionState {
                    mode: state.mode.clone(),
                    auto: match run_state {
                        MachineRun::Idle => match state.auto {
                            FumeExtractionAutomaticState::Idle => {
                                FumeExtractionAutomaticState::Idle
                            }
                            FumeExtractionAutomaticState::RunOn(t) => {
                                FumeExtractionAutomaticState::RunOn(t)
                            }
                            FumeExtractionAutomaticState::Demand => {
                                FumeExtractionAutomaticState::RunOn(Instant::now() + TIMEOUT)
                            }
                        },
                        MachineRun::Running => FumeExtractionAutomaticState::Demand,
                    },
                },
                Either4::Third(mode) => FumeExtractionState {
                    mode,
                    auto: state.auto.clone(),
                },
                Either4::Fourth(()) => {
                    debug!("Run on timer expired");
                    FumeExtractionState {
                        mode: state.mode.clone(),
                        auto: FumeExtractionAutomaticState::Idle,
                    }
                }
            }
        };

        info!("Fume extraction fan state {} -> {}", state, new_state);

        // Turn off demand relay if the machine is not powered.
        fan_tx.send(match machine_power {
            MachinePower::Off => FumeExtractionFan::Idle,
            MachinePower::On => (&new_state).into(),
        });

        state = new_state;
    }
}
