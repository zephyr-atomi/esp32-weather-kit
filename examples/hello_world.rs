#![no_std]
#![no_main]

use core::any::Any;
use hal::{clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*, Delay};
use esp_backtrace as _;
use esp_hal_common::gpio::{GpioPin, Output};
use log::{info, LevelFilter};
use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};
use stepper_driver::MotorDriver;
use esp_hal_common::gpio;
use esp_println::logger::init_logger;
use esp_println::println;

#[entry]
fn main() -> ! {
    init_logger(LevelFilter::Info);
    println!("Program started!");

    hal_exp::util::init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    info!("hello world 3");

    let mut t = false;
    loop {
        info!("Current: {}", t);
        t = !t;

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}