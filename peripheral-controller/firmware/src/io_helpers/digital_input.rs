use defmt::{debug, Format};
use embassy_rp::gpio::{Input, Level};

pub(crate) trait StateFromDigitalInputs<const N: usize> {
    fn from_inputs(inputs: [Level; N]) -> Self
    where
        Self: Sized;
}

pub(crate) struct DigitalInputStateChangeDetector<const N: usize, S> {
    detector: MultiPinChangeDetector<N>,
    _state_type: core::marker::PhantomData<S>,
}

impl<const N: usize, S: StateFromDigitalInputs<N> + Format> DigitalInputStateChangeDetector<N, S> {
    pub(crate) fn new(inputs: [Input<'static>; N]) -> Self {
        Self {
            detector: MultiPinChangeDetector::new(inputs),
            _state_type: core::marker::PhantomData,
        }
    }

    pub(crate) fn update(&mut self) -> Option<S> {
        match self.detector.update() {
            Some(inputs) => {
                let state = S::from_inputs(inputs);
                debug!("New state: {}", state);
                Some(state)
            }
            None => None,
        }
    }
}

pub(crate) struct MultiPinChangeDetector<const N: usize> {
    inputs: [Input<'static>; N],
    last: Option<[Level; N]>,
}

impl<const N: usize> MultiPinChangeDetector<N> {
    pub(crate) fn new(inputs: [Input<'static>; N]) -> Self {
        Self { inputs, last: None }
    }

    pub(crate) fn update(&mut self) -> Option<[Level; N]> {
        let mut new = [Level::Low; N];
        for (i, level) in new.iter_mut().enumerate().take(N) {
            *level = self.inputs[i].get_level();
        }

        let changed = self.last != Some(new);
        self.last = Some(new);

        if changed {
            self.last
        } else {
            None
        }
    }
}
