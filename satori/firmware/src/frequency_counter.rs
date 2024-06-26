
#[derive(Clone)]
pub(crate) struct FrequencyCounter {
    counts: u32,
}

impl FrequencyCounter {
    pub fn new() -> Self {
        Self {
            counts: 0,
        }
    }

    pub fn count(&mut self) {
        self.counts = self.counts.saturating_add(1);
    }

    pub fn measure(&mut self) -> f32 {
        let counts = self.counts;
        self.counts = 0;

        (counts / 2) as f32
    }
}
