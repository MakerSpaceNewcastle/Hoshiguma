#[macro_export]
macro_rules! input1 {
    ($avr:expr) => {
        $avr.pins().pd2()
    };
}

#[macro_export]
macro_rules! input2 {
    ($avr:expr) => {
        $avr.pins().pd3()
    };
}

#[macro_export]
macro_rules! input3 {
    ($avr:expr) => {
        $avr.pins().pd4()
    };
}

#[macro_export]
macro_rules! input4 {
    ($avr:expr) => {
        $avr.pins().pd5()
    };
}

#[macro_export]
macro_rules! input5 {
    ($avr:expr) => {
        $avr.pins().pd6()
    };
}

#[macro_export]
macro_rules! input6 {
    ($avr:expr) => {
        $avr.pins().pd7()
    };
}
