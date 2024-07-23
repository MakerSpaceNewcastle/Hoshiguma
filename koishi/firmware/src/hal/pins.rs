use atmega_hal::port::Pin;

avr_hal_generic::renamed_pins! {
    pub struct Pins {
        pub relay1: atmega_hal::port::PC3 = pc3, /// A3
        pub relay2: atmega_hal::port::PC2 = pc2, /// A2
        pub relay3: atmega_hal::port::PC1 = pc1, /// A1
        pub relay4: atmega_hal::port::PC0 = pc0, /// A0
        pub relay5: atmega_hal::port::PB4 = pb4, /// D12
        pub relay6: atmega_hal::port::PB3 = pb3, /// D11
        pub relay7: atmega_hal::port::PB2 = pb2, /// D10
        pub relay8: atmega_hal::port::PB1 = pb1, /// D9

        pub in1: atmega_hal::port::PD2 = pd2, /// D2
        pub in2: atmega_hal::port::PD3 = pd3, /// D3
        pub in3: atmega_hal::port::PD4 = pd4, /// D4
        pub in4: atmega_hal::port::PD5 = pd5, /// D5
        pub in5: atmega_hal::port::PD6 = pd6, /// D6
        pub in6: atmega_hal::port::PD7 = pd7, /// D7

        pub a4: atmega_hal::port::PC4 = pc4,
        pub a5: atmega_hal::port::PC5 = pc5,
        pub d0: atmega_hal::port::PD0 = pd0,
        pub d1: atmega_hal::port::PD1 = pd1,
        pub d8: atmega_hal::port::PB0 = pb0,
        pub d13: atmega_hal::port::PB5 = pb5,
    }

    impl Pins {
        type Pin = Pin;
        type McuPins = atmega_hal::Pins;
    }
}
