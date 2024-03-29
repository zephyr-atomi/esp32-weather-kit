#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_hal_common::systimer::SystemTimer;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, Rtc, IO};

use esp_hal_common::timer::TimerGroup;
use log::info;
use hal_exp::ets_delay::EtsDelay;
use hal_exp::util::init_heap;

#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    esp_println::logger::init_logger_from_env();

    let clocks = ClockControl::max(system.clock_control).freeze();
    // let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    info!("clocks: {:?}", clocks.xtal_clock);
    info!("clocks: {:?}", clocks.apb_clock);
    info!("clocks: {:?}", clocks.cpu_clock);

    let mut ets_delay = EtsDelay;

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    let sys_timer = SystemTimer::new(peripherals.SYSTIMER);
    info!("SYSTIMER Current value = {}", SystemTimer::now());

    let dht11_pin = io.pins.gpio9.into_open_drain_output();
    let mut dht11 = dht11::Dht11::new(dht11_pin);

    log::info!("Hello world!");
    let mut count = 0u32;

    let mut last_ts = 0u64;
    loop {
        clocks.xtal_clock.raw();
        let now = SystemTimer::now() - last_ts;
        // info!("Loop... {}, time: {}, delta: {}", count, now, now - last_ts);
        count += 1;
        last_ts = now;
        //
        let t = dht11.perform_measurement(&mut ets_delay);
        log::info!("measurements: {:?}", t);

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
