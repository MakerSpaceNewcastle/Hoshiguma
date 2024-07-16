use hoshiguma_foundational_data::{
    satori::{
        CoolantLevel, MachineProblem, MachineProblems, ObservedState, PotentialMachineProblems,
        ProblemKind, ProblemSeverity,
    },
    TimeMillis,
};

pub(super) struct RuleEvaluationContext<'a> {
    pub state: &'a ObservedState,
    pub now: TimeMillis,
    pub last_potential_problems: &'a PotentialMachineProblems,
    pub potential_problems: &'a mut PotentialMachineProblems,
    pub problems: &'a mut MachineProblems,
}

pub(super) fn evaluate(mut ctx: RuleEvaluationContext) {
    coolant_level(&mut ctx);
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
fn coolant_level(ctx: &mut RuleEvaluationContext) {
    let active_level_potential_problem = ctx
        .potential_problems
        .iter()
        .find(|i| i.problem.kind == ProblemKind::CoolantLevelInsufficient);

    // TODO
    match &ctx.state.coolant_level {
        Some(level) => match level {
            CoolantLevel::Full => {
                // Do nothing.
                // There is no problem, water is reported to be full.
            }
            CoolantLevel::Low => todo!(), // it depends
            CoolantLevel::CriticallyLow => {
                // The coolant tank is effectively (< a few inches) empty.
                // Add an active problem with critical severity.
                ctx.problems
                    .push(MachineProblem {
                        kind: ProblemKind::CoolantLevelInsufficient,
                        severity: ProblemSeverity::Critical,
                    })
                    .unwrap();
            }
        },
        None => todo!(), // it depends
    }
}
