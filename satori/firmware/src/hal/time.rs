use avr_device::interrupt::Mutex;
use core::cell::Cell;

pub(crate) type Delay = avr_hal_generic::delay::Delay<super::Clock>;

const PRESCALER: u32 = 1024;
const TIMER_COUNTS: u32 = 125;

const MILLIS_INCREMENT: u32 = PRESCALER * TIMER_COUNTS / 16000;

// static MILLIS_COUNTER: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static mut MILLIS_COUNTER_SHITE: u32 = 0;

pub(crate) fn millis_init(tc0: atmega_hal::pac::TC0) {
    // Configure the timer for the above interval (in CTC mode)
    // and enable its interrupt.
    tc0.tccr0a.write(|w| w.wgm0().ctc());
    tc0.ocr0a.write(|w| w.bits(TIMER_COUNTS as u8));
    tc0.tccr0b.write(|w| match PRESCALER {
        8 => w.cs0().prescale_8(),
        64 => w.cs0().prescale_64(),
        256 => w.cs0().prescale_256(),
        1024 => w.cs0().prescale_1024(),
        _ => panic!(),
    });
    tc0.timsk0.write(|w| w.ocie0a().set_bit());

    // Reset the global millisecond counter
    // avr_device::interrupt::free(|cs| {
    //     MILLIS_COUNTER.borrow(cs).set(0);
    // });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|_cs| {
        // TODO
        // let counter_cell = MILLIS_COUNTER.borrow(cs);
        // let counter = counter_cell.get();
        // let new_counter = counter.wrapping_add(MILLIS_INCREMENT);
        // counter_cell.set(new_counter);
        unsafe {
            MILLIS_COUNTER_SHITE = MILLIS_COUNTER_SHITE.wrapping_add(MILLIS_INCREMENT);
        }
    })
}

pub(crate) type TimeMillis = u32;

pub(crate) fn millis() -> TimeMillis {
    // TODO
    // avr_device::interrupt::free(|cs| MILLIS_COUNTER.borrow(cs).get())
    unsafe { MILLIS_COUNTER_SHITE }
}
