use super::debounce::DebouncerLevelExt;
use defmt::{debug, Format};
use embassy_rp::gpio::{Input, Level};

pub(crate) trait StateFromDigitalInputs<const N: usize> {
    fn from_inputs(inputs: [Level; N]) -> Self
    where
        Self: Sized;
}

pub(crate) struct DigitalInputStateChangeDetector<D, const N: usize, S> {
    detector: MultiPinChangeDetector<D, N>,
    _state_type: core::marker::PhantomData<S>,
}

impl<D: DebouncerLevelExt, const N: usize, S: StateFromDigitalInputs<N> + Format>
    DigitalInputStateChangeDetector<D, N, S>
{
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

pub(crate) struct MultiPinChangeDetector<D, const N: usize> {
    inputs: [Input<'static>; N],
    debouncers: [D; N],
    changed: bool,
}

impl<D: DebouncerLevelExt, const N: usize> MultiPinChangeDetector<D, N> {
    pub(crate) fn new(inputs: [Input<'static>; N]) -> Self {
        let debouncers = core::array::from_fn(|i| D::new(inputs[i].get_level()));
        Self {
            inputs,
            debouncers,
            changed: true,
        }
    }

    pub(crate) fn update(&mut self) -> Option<[Level; N]> {
        for (i, deb) in self.debouncers.iter_mut().enumerate().take(N) {
            self.changed =
                self.changed || (*deb).update_level(self.inputs[i].get_level()).is_some();
        }

        if self.changed {
            let mut levels = [Level::Low; N];
            for (i, level) in levels.iter_mut().enumerate() {
                *level = self.debouncers[i].get_level();
            }
            self.changed = false;
            Some(levels)
        } else {
            None
        }
    }
}
