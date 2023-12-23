#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_hal_common::systimer::SystemTimer;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO};

use esp_hal_common::timer::TimerGroup;
use heapless::spsc::Queue;
use log::info;
use hal_exp::ets_delay::EtsDelay;
use alloc::string::String;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};

const TIMEOUT_TICKS: u64 = 160_000_0;  // 100ms

#[entry]
fn main() -> ! {
    hal_exp::util::init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    let sys_timer = SystemTimer::new(peripherals.SYSTIMER);
    info!("SYSTIMER Current value = {}", SystemTimer::now());

    let mut dht11_pin = io.pins.gpio9.into_open_drain_output();

    esp_println::logger::init_logger_from_env();
    log::info!("Hello world!");
    let mut count = 0u32;

    let mut ets_delay = EtsDelay;
    loop {
        info!("Loop... {}, time: {}", count, SystemTimer::now());
        count += 1;
        // loop to read all data.
        let mut time_intervals = Queue::<u64, 100>::new();
        let mut last_time = SystemTimer::now();
        let mut last_state = false;

        let res = hal_exp::dht11::read_and_decode_dht11(&mut dht11_pin, &mut ets_delay);
        info!("Get DHT11 result: {:x?}", res);

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
