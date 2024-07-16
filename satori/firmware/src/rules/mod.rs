use hoshiguma_foundational_data::{
    satori::{CoolantLevel, MachineProblems, ObservedState, PotentialMachineProblems},
    TimeMillis,
};

pub(super) struct RuleEvaluationContext<'a> {
    pub state: &'a ObservedState,
    pub now: TimeMillis,
    pub last_potential_problems: &'a PotentialMachineProblems,
    pub potential_problems: &'a PotentialMachineProblems,
    pub problems: &'a MachineProblems,
}

pub(super) fn evaluate(ctx: &RuleEvaluationContext) {
    coolant_level(ctx);
    // TODO: coolant flow
    // TODO: pump RPM
    // TODO: temperatures
    // TODO: sensor fault
}

/// Ensure that the coolant level is sufficient.
///
/// Resolutions:
///     - full: no problems
///     - low: potential problem, promote to problem in 5 minutes
///     - empty: immediate problem
///     - unknown: potential problem, promote to problem in 5 minutes
fn coolant_level(ctx: &RuleEvaluationContext) {
    // TODO
    match &ctx.state.coolant_level {
        Some(level) => match level {
            CoolantLevel::Full => todo!(),
            CoolantLevel::Low => todo!(),
            CoolantLevel::CriticallyLow => todo!(),
        },
        None => todo!(),
    }
}
