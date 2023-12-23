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
    let start_time = SystemTimer::now();
    for _ in 0..10000 {
        delay.delay_us(1u16);
    }
    let end_time = SystemTimer::now();
    let delay_duration = end_time - start_time;
    info!("Delay duration: {}", delay_duration);

    let start_time = SystemTimer::now();
    for _ in 0..10000 {
        ets_delay.delay_us(1u16);
    }
    let end_time = SystemTimer::now();
    let ets_delay_duration = end_time - start_time;
    info!("EtsDelay duration: {}", ets_delay_duration);

    let start_time = SystemTimer::now();
    ets_delay.delay_us(10000u16);
    let end_time = SystemTimer::now();
    let ets_delay_duration = end_time - start_time;
    info!("2 EtsDelay duration: {}", ets_delay_duration);

    // 输出两种延迟的总时长

    loop {}
}
