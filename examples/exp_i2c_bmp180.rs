//! Read calibration data from BMP180 sensor
//!
//! This example dumps the calibration data from a BMP180 sensor
//!
//! The following wiring is assumed:
//! - SDA => GPIO1
//! - SCL => GPIO2

#![no_std]
#![no_main]

use hal::{clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*, Delay};
use esp_backtrace as _;
use hal_exp::bmp180::{BMP180, Oversampling};

#[entry]
fn main() -> ! {
    hal_exp::util::init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    esp_println::logger::init_logger_from_env();

    // Create a new peripheral object with the described wiring
    // and standard I2C clock speed
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio5,
        io.pins.gpio6,
        100u32.kHz(),
        &clocks,
    );

    let mut bmp180 = BMP180::new(i2c, delay).unwrap();

    loop {
        let (temp, pressure) = bmp180.temperature_and_pressure(Oversampling::O1).unwrap();
        log::info!("Temp: {:.2}C Pressure: {:.2}hPa", temp as f32 / 10.0, pressure as f32 / 100.0);

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
