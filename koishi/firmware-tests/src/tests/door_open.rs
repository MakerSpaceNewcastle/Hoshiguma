#[test]
fn door_open() {
    let mut avr = crate::avr();

    avr.run_for_ms(100);

    crate::door_interlock!(avr).set_low();
    crate::external_enable!(avr).set_high();
    crate::extractor_override!(avr).set_low();
    crate::machine_status!(avr).set_low();
    crate::air_assist_demand!(avr).set_low();

    avr.run_for_ms(10);

    crate::status_lamp_assert_red!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_open!(crate::controller_door_interlock!(avr));
    crate::relay_assert_open!(crate::laser_enable!(avr));
    crate::relay_assert_open!(crate::air_assist_compressor!(avr));
    crate::relay_assert_open!(crate::fume_extractor!(avr));

    crate::door_interlock!(avr).set_high();

    avr.run_for_ms(10);

    crate::status_lamp_assert_green!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_closed!(crate::controller_door_interlock!(avr));
    crate::relay_assert_closed!(crate::laser_enable!(avr));
    crate::relay_assert_open!(crate::air_assist_compressor!(avr));
    crate::relay_assert_open!(crate::fume_extractor!(avr));
}
