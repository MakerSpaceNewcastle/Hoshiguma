#[macro_export]
macro_rules! status_lamp_1 {
    ($avr:expr) => {
        $crate::relay8!($avr)
    };
}

#[macro_export]
macro_rules! status_lamp_2 {
    ($avr:expr) => {
        $crate::relay7!($avr)
    };
}

#[macro_export]
macro_rules! status_lamp_assert_red {
    ($avr:expr) => {
        $crate::relay_assert_open!($crate::status_lamp_1!($avr));
        $crate::relay_assert_open!($crate::status_lamp_2!($avr));
    };
}

#[macro_export]
macro_rules! status_lamp_assert_amber {
    ($avr:expr) => {
        $crate::relay_assert_open!($crate::status_lamp_1!($avr));
        $crate::relay_assert_closed!($crate::status_lamp_2!($avr));
    };
}

#[macro_export]
macro_rules! status_lamp_assert_green {
    ($avr:expr) => {
        $crate::relay_assert_closed!($crate::status_lamp_1!($avr));
        $crate::relay_assert_open!($crate::status_lamp_2!($avr));
    };
}

#[macro_export]
macro_rules! controller_door_interlock {
    ($avr:expr) => {
        $crate::relay6!($avr)
    };
}

#[macro_export]
macro_rules! controller_cooling_interlock {
    ($avr:expr) => {
        $crate::relay5!($avr)
    };
}

#[macro_export]
macro_rules! laser_enable {
    ($avr:expr) => {
        $crate::relay4!($avr)
    };
}

#[macro_export]
macro_rules! fume_extractor {
    ($avr:expr) => {
        $crate::relay2!($avr)
    };
}

#[macro_export]
macro_rules! air_assist_compressor {
    ($avr:expr) => {
        $crate::relay1!($avr)
    };
}
