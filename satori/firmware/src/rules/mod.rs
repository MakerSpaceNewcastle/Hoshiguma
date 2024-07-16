use hoshiguma_foundational_data::{
    satori::{MachineProblems, ObservedState, PotentialMachineProblems},
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
    // TODO
}

/// Ensure that the coolant level is sufficient.
fn coolant_level(ctx: &RuleEvaluationContext) {
    // TODO
}
