mod devices;
mod pins;
mod tests;

#[cfg(test)]
use avr_tester::*;

#[cfg(test)]
fn avr() -> AvrTester {
    AvrTester::atmega328p()
        .with_clock_of_8_mhz()
        .load("../firmware/target/avr-atmega328p/release/koishi.elf")
}
