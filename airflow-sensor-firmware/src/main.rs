#![no_std]
#![no_main]

// mod rpc;

use assign_resources::assign_resources;
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_protocol::types::{BootReason, SystemInformation};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::{
    embassy_rp::{
        gpio::{Level, Output},
        watchdog::Watchdog,
    },
    peripherals::{self, Peri, PicoPlc},
};

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    communication: CommunicationResources {
        uart: UART0,
        tx_pin: IO_0,
        rx_pin: IO_1,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { PicoPlc::steal() };
    let r = split_resources!(p);

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("{}", system_information());

    // TODO

    unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

    // unwrap!(spawner.spawn(rpc::task(r.communication, machine)));
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(600));

    loop {
        watchdog.feed();
        onboard_led.toggle();
        Timer::after_millis(500).await;
    }
}

fn system_information() -> SystemInformation {
    SystemInformation {
        git_revision: git_version::git_version!().try_into().unwrap(),
        last_boot_reason: boot_reason(),
        uptime_milliseconds: Instant::now().as_millis(),
    }
}

fn boot_reason() -> BootReason {
    let reason = pico_plc_bsp::embassy_rp::pac::WATCHDOG.reason().read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}
