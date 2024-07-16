use hoshiguma_foundational_data::{
    satori::{
        CoolantLevel, MachineProblem, MachineProblems, ObservedState, PotentialMachineProblem,
        PotentialMachineProblems, ProblemKind, ProblemSeverity,
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
    let sensor_potential_problem = ctx
        .potential_problems
        .iter()
        .find(|i| i.problem.kind == ProblemKind::CoolantLevelSensorFault);

    let level_potential_problem = ctx
        .potential_problems
        .iter()
        .find(|i| i.problem.kind == ProblemKind::CoolantLevelInsufficient);

    match &ctx.state.coolant_level {
        Some(level) => match level {
            CoolantLevel::Full => {
                // Do nothing.
                // There is no problem, water is reported to be full.
            }
            CoolantLevel::Low => {
                match level_potential_problem {
                    Some(problem) => {
                        // Low coolant level was a problem before and still is a problem.
                        // Check how long it has been a potential problem.
                        if ctx.now.wrapping_sub(problem.since) > 500 {
                            // Has been a potential problem long enough to now be an actual problem.
                            ctx.problems
                                .push(MachineProblem {
                                    kind: ProblemKind::CoolantLevelInsufficient,
                                    severity: ProblemSeverity::Critical,
                                })
                                .unwrap();
                        } else {
                            // Not long enough to be too concerned, yet...
                            // Just continue with the existing potential problem.
                            ctx.potential_problems.push(problem.clone()).unwrap();
                        }
                    }
                    None => {
                        // Low coolant level was not a problem before but now is.
                        // Add a new potential problem.
                        ctx.potential_problems
                            .push(PotentialMachineProblem {
                                problem: MachineProblem {
                                    kind: ProblemKind::CoolantLevelInsufficient,
                                    severity: ProblemSeverity::Critical,
                                },
                                since: ctx.now,
                            })
                            .unwrap();
                    }
                }
            }
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
        None => {
            // TODO
        }
    }
}
