use atmega_hal::port::Pin;

avr_hal_generic::renamed_pins! {
    pub struct Pins {
        pub led: atmega_hal::port::PB5 = pb5,

        pub machine_enable: atmega_hal::port::PD2 = pd2,

        pub rj45_pin7: atmega_hal::port::PD4 = pd4,
        pub rj45_pin6: atmega_hal::port::PD5 = pd5,
        pub rj45_pin5: atmega_hal::port::PD6 = pd6,
        pub rj45_pin4: atmega_hal::port::PD7 = pd7,
        pub rj45_pin3: atmega_hal::port::PB0 = pb0,
        pub rj45_pin2: atmega_hal::port::PB1 = pb1,

        pub d0: atmega_hal::port::PD0 = pd0,
        pub d1: atmega_hal::port::PD1 = pd1,
        pub d3: atmega_hal::port::PD3 = pd3,
        pub d10: atmega_hal::port::PB2 = pb2,
        pub d11: atmega_hal::port::PB3 = pb3,
        pub d12: atmega_hal::port::PB4 = pb4,

        pub a0: atmega_hal::port::PC0 = pc0,
        pub a1: atmega_hal::port::PC1 = pc1,
        pub a2: atmega_hal::port::PC2 = pc2,
        pub a3: atmega_hal::port::PC3 = pc3,
        pub a4: atmega_hal::port::PC4 = pc4,
        pub a5: atmega_hal::port::PC5 = pc5,
    }

    impl Pins {
        type Pin = Pin;
        type McuPins = atmega_hal::Pins;
    }
}
