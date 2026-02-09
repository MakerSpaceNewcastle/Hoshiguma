use defmt::{debug, info};
use embassy_time::{Duration, Instant};
use hoshiguma_api::{
    Severity,
    cooler::{CoolantPumpState, CoolantRate},
};
use hoshiguma_common::changed::ObservedValue;

crate::state_machine!(InputMessage, OutputMessage, State, 4);

pub enum InputMessage {
    CoolantPumpState(CoolantPumpState),
    RateFlow(CoolantRate),
    RateReturn(CoolantRate),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    RateSeverity(Severity),
    SymmetrySeverity(Severity),
}

pub struct State {
    pump_state: CoolantPumpState,
    pump_state_change: Instant,

    flow: Option<CoolantRate>,
    ret: Option<CoolantRate>,

    output_rate_severity: ObservedValue<Severity>,
    output_symmetry_severity: ObservedValue<Severity>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            pump_state: CoolantPumpState::Idle,
            pump_state_change: Instant::now(),

            flow: None,
            ret: None,

            output_rate_severity: ObservedValue::default(),
            output_symmetry_severity: ObservedValue::default(),
        }
    }
}

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            match self.input_channel.receive().await {
                InputMessage::CoolantPumpState(state) => {
                    self.state.pump_state = state;
                    self.state.pump_state_change = Instant::now();
                }
                InputMessage::RateFlow(rate) => {
                    self.state.flow = Some(rate);
                }
                InputMessage::RateReturn(rate) => {
                    self.state.ret = Some(rate);
                }
            }

            let severity = match self.state.flow {
                Some(rate) => flow_rate_to_severity(rate),
                None => Severity::Critical,
            };
            info!("flow rate {} = severity {}", self.state.flow, severity);
            self.state
                .output_rate_severity
                .update_and_async(severity, async |v| {
                    self.output_channel
                        .send(OutputMessage::RateSeverity(v))
                        .await;
                })
                .await;

            let severity = if let (Some(flow), Some(ret)) = (self.state.flow, self.state.ret) {
                let difference = flow - ret;
                debug!("difference {}", difference);
                let severity = rate_symmetry_to_severity(difference);

                if severity > Severity::Information
                    && self.state.pump_state == CoolantPumpState::Run
                    && (Instant::now() - self.state.pump_state_change) < PUMP_RUN_UP_TIME
                {
                    // Limit severity during the pump run up period
                    Severity::Information
                } else if self.state.pump_state == CoolantPumpState::Idle {
                    // Normal if the pump is off
                    Severity::Normal
                } else {
                    severity
                }
            } else {
                Severity::Critical
            };
            info!("symmetry severity {}", severity);
            self.state
                .output_symmetry_severity
                .update_and_async(severity, async |v| {
                    self.output_channel
                        .send(OutputMessage::SymmetrySeverity(v))
                        .await;
                })
                .await;
        }
    }
}

/// Time taken for the coolant flow to stabalise after the coolant pump has been started.
/// Taking into account flow and return rate sampling interval.
const PUMP_RUN_UP_TIME: Duration = Duration::from_secs(5);

fn flow_rate_to_severity(rate: CoolantRate) -> Severity {
    const WARN: CoolantRate = CoolantRate::new(4.5);
    const CRITICAL: CoolantRate = CoolantRate::new(2.0);

    if rate < CRITICAL {
        Severity::Critical
    } else if rate < WARN {
        Severity::Warning
    } else {
        Severity::Normal
    }
}

fn rate_symmetry_to_severity(difference: CoolantRate) -> Severity {
    const INFORMATION: CoolantRate = CoolantRate::new(0.1);
    const WARN: CoolantRate = CoolantRate::new(0.25);
    const FATAL: CoolantRate = CoolantRate::new(0.5);

    if difference > FATAL {
        Severity::Fatal
    } else if difference > WARN {
        Severity::Warning
    } else if difference > INFORMATION {
        Severity::Information
    } else {
        Severity::Normal
    }
}
