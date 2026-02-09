#![no_std]
#![no_main]

mod api;
mod devices;
mod hmi;
mod input_change_detector;
mod logic;
mod network;
mod remote_device_monitor;
mod self_telemetry;
mod telemetry;
mod telemetry_bridge_comm;
#[cfg(feature = "trace")]
mod trace;
mod wall_time;

use assign_resources::assign_resources;
use core::sync::atomic::Ordering;
use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::{Spawner, raw::Executor};
use embassy_rp::{
    Peri,
    clocks::ClockConfig,
    gpio::{Level, Output},
    multicore::{Stack, spawn_core1},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Ticker};
use hoshiguma_api::BootReason;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic::AtomicBool;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_19,
    },
    ethernet: EthernetResources {
        pio: PIO0,
        mosi: PIN_23,
        miso: PIN_22,
        sck: PIN_21,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,
        cs: PIN_20,
        int: PIN_24,
        reset: PIN_25,
    },
    onewire: OnewireResources {
        pio: PIO1,
        pin: PIN_28,
    },

    doors_detect: DoorsDetectResources {
        detect: PIN_0, // Input 8
    },
    machine_power_detect: AcBusPowerDetectResources {
        detect: PIN_1, // Input 7
    },
    air_assist_demand_detect: AirAssistDemandDetectResources {
        detect: PIN_4, // Input 6
    },
    machine_run_detect: MachineRunDetectResources {
        detect: PIN_3, // Input 5
    },

    machine_enable: MachineEnableResources {
        relay: PIN_8, // Relay 1
    },
    laser_enable: LaserEnableResources {
        relay: PIN_9, // Relay 2
    },
    air_assist_pump: AirAssistPumpResources {
        relay: PIN_11, // Relay 4
    },
    machine_power: MachinePowerResources {
        relay: PIN_12, // Relay 5
    },
    fume_extraction_fan: FumeExtractionFanResources {
        relay: PIN_13, // Relay 6
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use crate::devices::local::{
        laser_enable::LaserEnableOutput, machine_enable::MachineEnableOutput,
    };

    // Flag the panic, indicating that executors should stop scheduling work
    PANIC_HALT.store(true, Ordering::Relaxed);

    let p = unsafe { embassy_rp::Peripherals::steal() };
    let r = split_resources!(p);

    // Disable the machine and laser
    let mut laser_enable = LaserEnableOutput::new(r.laser_enable);
    laser_enable.set_safe();
    let mut machine_enable = MachineEnableOutput::new(r.machine_enable);
    machine_enable.set_safe();

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed(Duration::from_millis(500));

        // Keep setting the enable outputs.
        // Not strictly needed, as no other tasks should be using the outputs at this point, but
        // here for belt and braces.
        laser_enable.set_safe();
        machine_enable.set_safe();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

static mut CORE_1_STACK: Stack<4096> = Stack::new();

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

static PANIC_HALT: AtomicBool = AtomicBool::new(false);

#[cortex_m_rt::entry]
fn main() -> ! {
    let config = embassy_rp::config::Config::new(ClockConfig::system_freq(200_000_000).unwrap());
    let p = embassy_rp::init(config);
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    // Core 1 deals with everything...
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE_1_STACK) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            #[cfg(feature = "trace")]
            trace::identify_core_1_executor(executor_1.id() as u32);
            let spawner = executor_1.spawner();

            spawner.spawn(watchdog_feed_task(r.status).unwrap());

            spawner.spawn(self_telemetry::task().unwrap());

            // Device drivers
            spawner.spawn(
                devices::local::ac_bus_power_detector::task(r.machine_power_detect).unwrap(),
            );
            spawner.spawn(
                devices::local::air_assist_demand_detector::task(r.air_assist_demand_detect)
                    .unwrap(),
            );
            spawner.spawn(devices::local::air_assist_pump::task(r.air_assist_pump).unwrap());
            spawner.spawn(devices::local::doors_detector::task(r.doors_detect).unwrap());
            spawner
                .spawn(devices::local::fume_extraction_fan::task(r.fume_extraction_fan).unwrap());
            spawner.spawn(devices::local::laser_enable::task(r.laser_enable).unwrap());
            spawner.spawn(devices::local::machine_enable::task(r.machine_enable).unwrap());
            spawner.spawn(devices::local::machine_power::task(r.machine_power).unwrap());
            spawner
                .spawn(devices::local::machine_run_detector::task(r.machine_run_detect).unwrap());
            spawner.spawn(devices::local::temperature_sensors::task(r.onewire).unwrap());

            // Logic
            logic::air_assist::init(spawner);
            logic::coolant_rate::init(spawner);
            logic::cooling::init(spawner);
            logic::extraction_airflow::init(spawner);
            logic::fume_extraction::init(spawner);
            logic::interlock::init(spawner);
            logic::hmi_status_screen::init(spawner);
            logic::machine_power::init(spawner);
            logic::status_light::init(spawner);
            logic::temperatures::init(spawner);

            #[cfg(feature = "test-panic-on-core-1")]
            spawner.spawn(dummy_panic().unwrap());

            loop {
                cortex_m::asm::wfe();
                if !PANIC_HALT.load(Ordering::Relaxed) {
                    unsafe { executor_1.poll() };
                }
            }
        },
    );

    // ...except networking, which is on core 0
    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    #[cfg(feature = "trace")]
    trace::identify_core_0_executor(executor_0.id() as u32);
    let spawner = executor_0.spawner();

    spawner.spawn(network_tasks(spawner, r.ethernet).unwrap());

    #[cfg(feature = "trace")]
    spawner.spawn(trace::task().unwrap());

    #[cfg(feature = "test-panic-on-core-0")]
    spawner.spawn(dummy_panic().unwrap());

    loop {
        cortex_m::asm::wfe();
        if !PANIC_HALT.load(Ordering::Relaxed) {
            unsafe { executor_0.poll() };
        }
    }
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("watchdog").await;

    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(500));

    let mut feed_ticker = Ticker::every(Duration::from_millis(100));

    loop {
        let start = Instant::now();

        // Blink LED
        onboard_led.toggle();

        // at 1 Hz
        for _ in 0..10 {
            watchdog.feed(Duration::from_millis(500));
            feed_ticker.next().await;
        }

        let end = Instant::now();
        let duration = end - start;
        if duration > Duration::from_millis(1050) {
            warn!(
                "WDT feed loop took a suspicious amount of time: {}",
                duration
            );
        }
    }
}

#[embassy_executor::task]
async fn network_tasks(spawner: Spawner, r: EthernetResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("network init").await;

    let net_stack = network::init(spawner, r).await;

    spawner.spawn(wall_time::task(net_stack).unwrap());
    spawner.spawn(telemetry::task(net_stack).unwrap());

    spawner.spawn(remote_device_monitor::task(net_stack).unwrap());

    spawner.spawn(devices::remote::state::task(net_stack).unwrap());
    spawner.spawn(devices::remote::observations::task(net_stack).unwrap());

    spawner.spawn(api::task(net_stack).unwrap());

    spawner.spawn(hmi::task(net_stack).unwrap());
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

fn boot_reason() -> BootReason {
    let reason = embassy_rp::pac::WATCHDOG.reason().read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}

#[macro_export]
macro_rules! variable_watch {
    ($name:ident, $type:ty, $receivers:expr) => {
        paste::paste! {
            static [<$name:upper>]: embassy_sync::watch::Watch<embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, $type, $receivers> = embassy_sync::watch::Watch::new();

            pub(crate) fn [<$name _rx>]() -> embassy_sync::watch::Receiver<'static, embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, $type, $receivers> {
                [<$name:upper>].receiver().unwrap()
            }
        }
    };
}
