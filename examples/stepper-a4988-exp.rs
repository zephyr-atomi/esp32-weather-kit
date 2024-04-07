//! Read calibration data from BMP180 sensor
//!
//! This example dumps the calibration data from a BMP180 sensor
//!
//! The following wiring is assumed:
//! - SDA => GPIO1
//! - SCL => GPIO2

#![no_std]
#![no_main]

use core::any::Any;

use esp_backtrace as _;
use esp_hal_common::gpio::{GpioPin, Output};
use esp_hal_common::gpio;
use esp_println::logger::init_logger;
use esp_println::println;
use hal::{clock::ClockControl, Delay, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*};
use log::{info, LevelFilter};
use stepper_driver::MotorDriver;

use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};

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

    esp_println::logger::init_logger_from_env();
    let dir_pin = io.pins.gpio2.into_push_pull_output();
    let step_pin = io.pins.gpio3.into_push_pull_output();

    info!("hello world 1");
    let mut driver = MotorDriver::a4988(Delay::new(&clocks), dir_pin, step_pin, 200, 1, 100f32);

    loop {
        driver.set_speed(100f32);
        driver.set_direction(true);
        driver.move_instant(20);
        driver.set_direction(false);
        driver.move_instant(10);

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
