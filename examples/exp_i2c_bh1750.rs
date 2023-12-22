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
use log::info;
use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};

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
        io.pins.gpio1,
        io.pins.gpio2,
        100u32.kHz(),
        &clocks,
    );

    let mut bh1750 = BH1750::new(i2c, delay);

    loop {

        info!("HiResMode 1.0 lx resolution");
        bh1750.set_resolution(Resolution::Lx1_0);
        for _ in 0..5 {
            info!("{:>6} lx", bh1750.illuminance().unwrap());
            delay.delay(1);
        }

        info!("HiResMode2 0.5 lx resolution");
        bh1750.set_resolution(Resolution::Lx0_5);

        for _ in 0..5 {
            info!("{:>6} lx", bh1750.illuminance().unwrap());
            delay.delay(1);
        }

        info!("LowResMode 4.0 lx resolution");
        bh1750.set_resolution(Resolution::Lx4_0);
        for _ in 0..5 {
            info!("{:>6} lx", bh1750.illuminance().unwrap());
            delay.delay(1);
        }

        info!("HiResMode 1.85 lx resolution and min measurement time (31)");
        bh1750.set_resolution(Resolution::Lx1_0);
        bh1750.set_measurement_time(MeasurementTime::Custom(31)).unwrap();

        for _ in 0..5 {
            info!("{:>6} lx", bh1750.illuminance().unwrap());
            delay.delay(1);
        }

        info!("HiResMode 0.23 lx resolution and max measurement time (254)");
        bh1750.set_resolution(Resolution::Lx1_0);
        bh1750.set_measurement_time(MeasurementTime::Custom(254)).unwrap();

        for _ in 0..5 {
            info!("{:>6} lx", bh1750.illuminance().unwrap());
            delay.delay(1);
        }

        info!("HiResMode2 0.11 lx resolution and max measurement time (254)");
        bh1750.reset().unwrap();
        bh1750.set_resolution(Resolution::Lx0_5);
        bh1750.set_measurement_time(MeasurementTime::Custom(254)).unwrap();

        for _ in 0..5 {
            info!("{:>6} lx", bh1750.illuminance().unwrap());
            delay.delay(1);
        }

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
