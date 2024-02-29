use esp_idf_hal::gpio::{InterruptType, PinDriver};
use esp_idf_sys as _;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

#[derive(Clone)]
pub(crate) struct FrequencyCounter {
    counts: Arc<AtomicU32>,
    counts_per_x: f32,
}

impl FrequencyCounter {
    pub fn new<A: esp_idf_hal::gpio::Pin, B: esp_idf_hal::gpio::InputMode>(
        pin: &mut PinDriver<A, B>,
        counts_per_x: f32,
    ) -> Self {
        pin.set_interrupt_type(InterruptType::PosEdge).unwrap();

        let counts = Arc::new(AtomicU32::new(0));

        {
            let counts = counts.clone();
            unsafe {
                pin.subscribe(move || {
                    counts.fetch_add(1, Ordering::Relaxed);
                })
                .unwrap();
            }
        }

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
