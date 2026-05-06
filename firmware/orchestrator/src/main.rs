#![no_std]
#![no_main]

// mod devices;
// mod logic;
mod network;
mod polled_input;
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
    gpio::{Level, Output},
    multicore::{Stack, spawn_core1},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Ticker};
use hoshiguma_api::{BootReason, COOLER_IP_ADDRESS};
use hoshiguma_common::remote_device_healthcheck::RemoteDeviceHealthCheck;
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
    machine_power_detect: MachinePowerDetectResources {
        detect: PIN_5, // Input 1
    },
    chassis_intrusion_detect: ChassisIntrusionDetectResources {
        detect: PIN_6, // Input 2
    },
    air_assist_demand_detect: AirAssistDemandDetectResources {
        detect: PIN_7, // Input 3
    },
    machine_run_detect: MachineRunDetectResources {
        detect: PIN_4, // Input 3
    },
    air_assist_pump: AirAssistPumpResources {
        relay: PIN_15, // Relay 8
    },
    fume_extraction_fan: FumeExtractionFanResources {
        relay: PIN_14, // Relay 7
    },
    laser_enable: LaserEnableResources {
        relay: PIN_13, // Relay 6
    },
    machine_enable: MachineEnableResources {
        relay: PIN_12, // Relay 5
    },
    machine_power: MachinePowerResources {
        relay: PIN_11, // Relay 4
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use crate::devices::{laser_enable::LaserEnableOutput, machine_enable::MachineEnableOutput};

    // Flag the panic, indicating that executors should stop scheduling work
    PANIC_HALT.store(true, Ordering::Relaxed);

    let p = unsafe { embassy_rp::Peripherals::steal() };
    let r = split_resources!(p);

    // Disable the machine and laser
    let mut laser_enable = LaserEnableOutput::new(r.laser_enable);
    laser_enable.set_panic();
    let mut machine_enable = MachineEnableOutput::new(r.machine_enable);
    machine_enable.set_panic();

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed(Duration::from_millis(500));

        // Keep setting the enable and status lamp outputs.
        // Not strictly needed, as no other tasks should be using the outputs at this point, but
        // here for belt and braces.
        laser_enable.set_panic();
        machine_enable.set_panic();

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
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    // Safety critical things go on core 1
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE_1_STACK) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            #[cfg(feature = "trace")]
            trace::identify_core_1_executor(executor_1.id() as u32);
            let spawner = executor_1.spawner();

            spawner.spawn(watchdog_feed_task(r.status).unwrap());

            // spawner.spawn(devices::machine_power_detector::task(r.machine_power_detect).unwrap());
            // spawner.spawn(devices::machine_run_detector::task(r.machine_run_detect).unwrap());
            // spawner.spawn(
            //     devices::chassis_intrusion_detector::task(r.chassis_intrusion_detect).unwrap(),
            // );

            // State monitor tasks
            // spawner.spawn(logic::safety::monitor::chassis_intrusion::task().unwrap());

            // State monitor observation and alarm tasks
            // spawner.spawn(logic::safety::monitor::observation_task().unwrap());
            // spawner.spawn(logic::safety::lockout::alarm_evaluation_task().unwrap());

            // Machine operation permission control tasks
            // spawner.spawn(logic::safety::lockout::machine_lockout_task().unwrap());
            // spawner.spawn(devices::laser_enable::task(r.laser_enable).unwrap());
            // spawner.spawn(devices::machine_enable::task(r.machine_enable).unwrap());

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

    // Everything else goes on core 0
    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    #[cfg(feature = "trace")]
    trace::identify_core_0_executor(executor_0.id() as u32);
    let spawner = executor_0.spawner();

    spawner.spawn(network_tasks(spawner, r.ethernet).unwrap());

    spawner.spawn(self_telemetry::task().unwrap());

    // spawner.spawn(logic::status_lamp::task().unwrap());
    // spawner.spawn(devices::status_lamp::task(r.status_lamp).unwrap());

    // spawner.spawn(devices::temperature_sensors::task(r.onewire).unwrap());
    // spawner.spawn(devices::accessories::task(r.accessories_bus).unwrap());

    // spawner.spawn(
    //     quick_and_dirty_machine_power_for_new_access_control_task(
    //         r.access_control,
    //         r.machine_power,
    //     )
    //     .unwrap(),
    // );

    // State monitor tasks
    // spawner.spawn(logic::safety::monitor::power::task().unwrap());
    // spawner.spawn(logic::safety::monitor::coolant_flow::task().unwrap());
    // spawner.spawn(logic::safety::monitor::coolant_level::task().unwrap());
    // spawner.spawn(logic::safety::monitor::extraction_airflow::task().unwrap());
    // spawner.spawn(logic::safety::monitor::temperatures_a::task().unwrap());
    // spawner.spawn(logic::safety::monitor::temperatures_b::task().unwrap());

    // Air assist control tasks
    // spawner.spawn(devices::air_assist_demand_detector::task(r.air_assist_demand_detect).unwrap());
    // spawner.spawn(devices::air_assist_pump::task(r.air_assist_pump).unwrap());
    // spawner.spawn(logic::air_assist::task().unwrap());

    // Fume extraction control tasks
    // spawner
    //     .spawn(devices::fume_extraction_mode_switch::task(r.fume_extraction_mode_switch).unwrap());
    // spawner.spawn(devices::fume_extraction_fan::task(r.fume_extraction_fan).unwrap());
    // spawner.spawn(logic::fume_extraction::task().unwrap());

    // Cooler control tasks
    // spawner.spawn(logic::cooling::control::task().unwrap());
    // spawner.spawn(logic::cooling::demand::task().unwrap());

    // Task reporting
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
    trace::name_task("wdt feed").await;

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
    let net_stack = network::init(spawner, r).await;
    spawner.spawn(wall_time::task(net_stack).unwrap());
    spawner.spawn(telemetry::task(net_stack).unwrap());

    // TODO
    spawner.spawn(cooler_monitor_task(net_stack).unwrap());
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

#[embassy_executor::task]
async fn cooler_monitor_task(stack: embassy_net::Stack<'static>) -> ! {
    let obs = RemoteDeviceHealthCheck::<
        hoshiguma_api::cooler::Request,
        hoshiguma_api::cooler::Response,
        _,
        _,
    >::new(
        stack,
        "cooler",
        COOLER_IP_ADDRESS,
        async |severity| {
            // TODO
            info!("TODO cooler severity: {}", severity);
        },
        |telem_str| {
            // TODO
            info!("TODO cooler telemetry: {}", telem_str.unwrap());
        },
    );

    obs.run().await
}
