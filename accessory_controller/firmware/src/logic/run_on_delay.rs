use core::ops::Add;
use ufmt::derive::uDebug;

#[derive(uDebug, Clone, PartialEq)]
enum State<T: PartialEq> {
    Demand,
    RunOn { end: T },
    Idle,
}

#[derive(uDebug, Clone, PartialEq)]
pub(crate) struct RunOnDelay<T: PartialEq> {
    delay: T,
    state: State<T>,
}

impl<T: Copy + PartialEq + PartialOrd + Add<Output = T>> RunOnDelay<T> {
    pub(crate) fn new(delay: T) -> Self {
        Self {
            delay,
            state: State::Idle,
        }
    }

    pub(crate) fn update(&mut self, time: T, demand: bool) {
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

    pub(crate) fn should_run(&self) -> bool {
        self.state != State::Idle
    }
}
