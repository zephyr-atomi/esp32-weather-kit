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
use hal::{clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*, Delay};
use esp_backtrace as _;
use esp_hal_common::gpio::{GpioPin, Output};
use log::info;
use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};
use stepper_driver::MotorDriver;
use esp_hal_common::gpio;

#[entry]
fn main() -> ! {
    hal_exp::util::init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let mut io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    // let mut t: esp_hal_common::gpio::AnyPin<Output<gpio::PushPull>> = io.pins.gpio19.into_push_pull_output().into();
    // t.set_low().unwrap();

    esp_println::logger::init_logger_from_env();
    let dir_pin : esp_hal_common::gpio::AnyPin<Output<gpio::PushPull>> = io.pins.gpio19.into_push_pull_output().into();
    let step_pin: esp_hal_common::gpio::AnyPin<Output<gpio::PushPull>> = io.pins.gpio18.into_push_pull_output().into();

    // // io.pins.gpio19.set_to_open_drain_output();
    let mut driver = MotorDriver::a4988(Delay::new(&clocks), dir_pin, step_pin, 32, 1, 60.0);

    info!("hello world");

    loop {
    }
}
