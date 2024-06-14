use hoshiguma_foundational_data::koishi::run_on_delay::{RunOnDelay, State};

pub(crate) trait RunOnDelayExt<T> {
    fn new(delay: T) -> Self;
    fn update(&mut self, time: T, demand: bool);
    fn should_run(&self) -> bool;
}

impl<T: PartialEq + PartialOrd + core::ops::Add<Output = T> + Copy> RunOnDelayExt<T>
    for RunOnDelay<T>
{
    fn new(delay: T) -> Self {
        Self {
            delay,
            state: State::Idle,
        }
    }

    fn update(&mut self, time: T, demand: bool) {
        self.state = if demand {
            State::Demand
        } else {
            match &self.state {
                State::Demand => State::RunOn {
                    end: time + self.delay,
                },
                State::RunOn { end } => {
                    if time > *end {
                        State::Idle
                    } else {
                        State::RunOn { end: *end }
                    }
                }

                State::Idle => State::Idle,
            }
        };
    }

    fn should_run(&self) -> bool {
        self.state != State::Idle
    }
}
