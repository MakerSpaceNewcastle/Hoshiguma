mod coolant_level;

use hoshiguma_foundational_data::{
    satori::{MachineProblems, ObservedState, PotentialMachineProblems},
    TimeMillis,
};

pub(super) struct RuleEvaluationContext<'a> {
    pub state: &'a ObservedState,
    pub now: TimeMillis,
    #[allow(dead_code)]
    pub last_potential_problems: &'a PotentialMachineProblems,
    pub potential_problems: &'a mut PotentialMachineProblems,
    pub problems: &'a mut MachineProblems,
}

pub(super) fn evaluate(mut ctx: RuleEvaluationContext) {
    self::coolant_level::coolant_level(&mut ctx);
    // TODO: coolant flow
    // TODO: pump RPM
    // TODO: temperatures
    // TODO: sensor fault
}
