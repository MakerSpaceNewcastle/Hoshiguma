#[test]
fn cycle() {
    let mut avr = crate::avr();

    avr.run_for_ms(100);

    crate::door_interlock!(avr).set_high();
    crate::extractor_override!(avr).set_low();
    crate::machine_status!(avr).set_low();
    crate::air_assist_demand!(avr).set_low();

    avr.run_for_ms(10);

    crate::status_lamp_assert_green!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_closed!(crate::controller_door_interlock!(avr));
    crate::relay_assert_closed!(crate::laser_enable!(avr));
    crate::relay_assert_open!(crate::air_assist_compressor!(avr));
    crate::relay_assert_open!(crate::fume_extractor!(avr));

    crate::machine_status!(avr).set_high();
    crate::air_assist_demand!(avr).set_high();

    avr.run_for_ms(10);

    crate::status_lamp_assert_amber!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_closed!(crate::controller_door_interlock!(avr));
    crate::relay_assert_closed!(crate::laser_enable!(avr));
    crate::relay_assert_closed!(crate::air_assist_compressor!(avr));
    crate::relay_assert_closed!(crate::fume_extractor!(avr));

    avr.run_for_ms(300);

    crate::air_assist_demand!(avr).set_low();

    avr.run_for_ms(10);

    crate::status_lamp_assert_amber!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_closed!(crate::controller_door_interlock!(avr));
    crate::relay_assert_closed!(crate::laser_enable!(avr));
    crate::relay_assert_open!(crate::air_assist_compressor!(avr));
    crate::relay_assert_closed!(crate::fume_extractor!(avr));

    avr.run_for_ms(300);

    crate::machine_status!(avr).set_low();

    avr.run_for_ms(10);

    crate::status_lamp_assert_green!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_closed!(crate::controller_door_interlock!(avr));
    crate::relay_assert_closed!(crate::laser_enable!(avr));
    crate::relay_assert_open!(crate::air_assist_compressor!(avr));
    crate::relay_assert_closed!(crate::fume_extractor!(avr));

    avr.run_for_ms(10);
    avr.run_for_ms(500);

    crate::status_lamp_assert_green!(avr);
    crate::relay_assert_closed!(crate::controller_cooling_interlock!(avr));
    crate::relay_assert_closed!(crate::controller_door_interlock!(avr));
    crate::relay_assert_closed!(crate::laser_enable!(avr));
    crate::relay_assert_open!(crate::air_assist_compressor!(avr));
    crate::relay_assert_open!(crate::fume_extractor!(avr));
}
