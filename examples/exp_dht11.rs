#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_hal_common::systimer::SystemTimer;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO};
use hal_exp::dht11::Dht11;

use esp_hal_common::timer::TimerGroup;
use log::info;


#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    let sys_timer = SystemTimer::new(peripherals.SYSTIMER);
    info!("SYSTIMER Current value = {}", SystemTimer::now());

    let dht11_pin = io.pins.gpio2.into_open_drain_output();
    let mut dht11 = Dht11::new(dht11_pin);

    esp_println::logger::init_logger_from_env();
    log::info!("Hello world!");
    let mut count = 0u32;

    loop {

        info!("Loop... {}, time: {}", count, SystemTimer::now());
        count += 1;

        let t = dht11.perform_measurement(&mut delay);
        log::info!("measurements: {:?}", t);

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
