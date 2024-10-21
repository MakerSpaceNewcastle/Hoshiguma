use defmt::{debug, Format};
use embassy_rp::gpio::{Level, Output};

pub(crate) trait StateToDigitalOutputs<const N: usize> {
    fn to_outputs(self) -> [Level; N];
}

pub(crate) struct DigitalOutputController<const N: usize, S> {
    outputs: [Output<'static>; N],
    _state_type: core::marker::PhantomData<S>,
}

impl<const N: usize, S: StateToDigitalOutputs<N> + Format> DigitalOutputController<N, S> {
    pub(crate) fn new(outputs: [Output<'static>; N]) -> Self {
        Self {
            outputs,
            _state_type: core::marker::PhantomData,
        }
    }

    pub(crate) fn set(&mut self, state: S) {
        debug!("Setting state {}", state);

        let levels = state.to_outputs();
        for (i, output) in self.outputs.iter_mut().enumerate().take(N) {
            output.set_level(levels[i]);
        }
    }
}
