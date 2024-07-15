use hoshiguma_foundational_data::{
    satori::{MachineProblems, ObservedState, PotentialMachineProblems},
    TimeMillis,
};

pub(super) fn evaluate_rules(
    state: &ObservedState,
    time: TimeMillis,
    last_potential_problems: &PotentialMachineProblems,
    potential_problems: &PotentialMachineProblems,
    problems: &MachineProblems,
) {
    // TODO
}
