#[macro_export]
macro_rules! relay_assert_open {
    ($pin:expr) => {
        $pin.assert_high()
    };
}

#[macro_export]
macro_rules! relay_assert_closed {
    ($pin:expr) => {
        $pin.assert_low()
    };
}

#[macro_export]
macro_rules! relay1 {
    ($avr:expr) => {
        $avr.pins().pc3()
    };
}

#[macro_export]
macro_rules! relay2 {
    ($avr:expr) => {
        $avr.pins().pc2()
    };
}

#[macro_export]
macro_rules! relay3 {
    ($avr:expr) => {
        $avr.pins().pc1()
    };
}

#[macro_export]
macro_rules! relay4 {
    ($avr:expr) => {
        $avr.pins().pc0()
    };
}

#[macro_export]
macro_rules! relay5 {
    ($avr:expr) => {
        $avr.pins().pb4()
    };
}

#[macro_export]
macro_rules! relay6 {
    ($avr:expr) => {
        $avr.pins().pb3()
    };
}

#[macro_export]
macro_rules! relay7 {
    ($avr:expr) => {
        $avr.pins().pb2()
    };
}

#[macro_export]
macro_rules! relay8 {
    ($avr:expr) => {
        $avr.pins().pb1()
    };
}
