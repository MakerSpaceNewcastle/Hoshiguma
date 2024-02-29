use super::State;

pub(super) fn lower_threshold<T: PartialOrd>(
    value: &T,
    warn: &Option<T>,
    critical: &Option<T>,
) -> State {
    if let Some(critical) = critical {
        if value < critical {
            return State::Critical;
        }
    }

    if let Some(warn) = warn {
        if value < warn {
            return State::Warn;
        }
    }

    State::Normal
}

pub(super) fn upper_threshold<T: PartialOrd>(
    value: &T,
    warn: &Option<T>,
    critical: &Option<T>,
) -> State {
    if let Some(critical) = critical {
        if value > critical {
            return State::Critical;
        }
    }

    if let Some(warn) = warn {
        if value > warn {
            return State::Warn;
        }
    }

    State::Normal
}
