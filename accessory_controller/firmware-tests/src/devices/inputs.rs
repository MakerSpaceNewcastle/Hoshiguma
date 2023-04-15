#[macro_export]
macro_rules! door_interlock {
    ($avr:expr) => {
        $crate::input1!($avr)
    };
}

#[macro_export]
macro_rules! extractor_override {
    ($avr:expr) => {
        $crate::input2!($avr)
    };
}

#[macro_export]
macro_rules! machine_status {
    ($avr:expr) => {
        $crate::input4!($avr)
    };
}

#[macro_export]
macro_rules! air_assist_demand {
    ($avr:expr) => {
        $crate::input5!($avr)
    };
}
