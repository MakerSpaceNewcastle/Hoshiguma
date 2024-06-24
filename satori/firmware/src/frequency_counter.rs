use core::sync::atomic::AtomicU32;

#[derive(Clone)]
pub(crate) struct FrequencyCounter {
    counts: Arc<AtomicU32>,
    counts_per_x: f32,
}

impl FrequencyCounter {
    pub fn new
        (
        counts_per_x: f32,
    ) -> Self {
        let counts = Arc::new(AtomicU32::new(0));

        Self {
            counts,
            counts_per_x,
        }
    }

    pub fn measure(&self) -> f32 {
        let counts = self.counts.fetch_and(0, Ordering::Relaxed) as f32;
        counts / self.counts_per_x
    }
}
