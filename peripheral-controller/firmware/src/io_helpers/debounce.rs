use debouncr::{DebouncerStateful, Edge, Repeat2};
use embassy_rp::gpio::Level;

pub(crate) trait DebouncerLevelExt {
    fn new(initial: Level) -> Self;
    fn update_level(&mut self, level: Level) -> Option<Edge>;
    fn get_level(&self) -> Level;
}

impl DebouncerLevelExt for DebouncerStateful<u8, Repeat2> {
    fn new(initial: Level) -> Self {
        debouncr::debounce_stateful_2(initial == Level::High)
    }

    fn update_level(&mut self, level: Level) -> Option<Edge> {
        self.update(level == Level::High)
    }

    fn get_level(&self) -> Level {
        if self.is_high() {
            Level::High
        } else {
            Level::Low
        }
    }
}
