//! I2C Display example
//!
//! This example prints some text on an SSD1306-based
//! display (via I2C)
//!
//! The following wiring is assumed:
//! - SDA => GPIO3
//! - SCL => GPIO4

#![no_std]
#![no_main]

extern crate alloc;
use embedded_graphics::{
    mono_font::{
        ascii::{FONT_6X10, FONT_9X18_BOLD},
        MonoTextStyleBuilder,
    },
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Alignment, Text},
};
use hal::{
    Delay,
    clock::ClockControl,
    gpio::IO,
    i2c::I2C,
    peripherals::Peripherals,
    prelude::*,
    timer::TimerGroup,
};
use esp_backtrace as _;
use nb::block;
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use log::{error, info};
use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};
use hal_exp::bmp180::{BMP180, Oversampling};
use alloc::format;
use hal_exp::shared_i2c::SharedI2cBus;

const DEFAULT_HUMIDITY:u16 = 450;

#[entry]
fn main() -> ! {
    hal_exp::util::init_heap();

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    esp_println::logger::init_logger_from_env();

    let timer_group0 = TimerGroup::new(peripherals.TIMG0, &clocks);
    let mut timer0 = timer_group0.timer0;

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Prepare I2C display
    let i2c = I2C::new(
        peripherals.I2C0,
        io.pins.gpio1,
        io.pins.gpio2,
        100u32.kHz(),
        &clocks,
    );
    let shared_i2c = SharedI2cBus::new(i2c);
    let interface = I2CDisplayInterface::new(shared_i2c.clone());
    let mut display = Ssd1306::new(interface, DisplaySize128x64, DisplayRotation::Rotate0)
        .into_buffered_graphics_mode();
    display.init().unwrap();

    // Specify different text styles
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On)
        .build();
    let text_style_big = MonoTextStyleBuilder::new()
        .font(&FONT_9X18_BOLD)
        .text_color(BinaryColor::On)
        .build();

    let mut bh1750 = BH1750::new(shared_i2c.clone(), delay);
    bh1750.set_measurement_time(MeasurementTime::Default).unwrap();
    bh1750.reset().unwrap();

    let mut bmp180 = BMP180::new(shared_i2c.clone(), delay).unwrap();

    let dht11_pin = io.pins.gpio9.into_open_drain_output();
    let mut dht11 = dht11::Dht11::new(dht11_pin);
    let mut ets_delay = hal_exp::ets_delay::EtsDelay;

    // Start timer (5 second interval)
    timer0.start(5u64.secs());
    loop {
        bh1750.set_resolution(Resolution::Lx1_0);
        let illuminance = bh1750.illuminance();
        let (temp, pressure) = bmp180.temperature_and_pressure(Oversampling::O1).unwrap();

        log::info!("Temp: {:.2}C Pressure: {:.2}kPa", temp as f32 / 10.0, pressure as f32 / 1000.0);

        let humidity = match dht11.perform_measurement(&mut ets_delay) {
            Ok(res) => res.humidity,
            Err(err) => {
                error!("DHT error: {:?}", err);
                DEFAULT_HUMIDITY
            }
        };

        // Fill display bufffer with a centered text with two lines (and two text
        // styles)
        let t = display.bounding_box();
        Text::with_alignment(
            format!("Humidity: {:.1}C", humidity as f32 / 10.0).as_str(),
            t.center() + Point::new(0, -8),
            text_style,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        Text::with_alignment(
            format!("Temp: {:.2}C", temp as f32 / 10.0).as_str(),
            t.center() + Point::new(0, 2),
            text_style,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        Text::with_alignment(
            format!("Pressure: {:.2}atm", pressure as f32 / 101300.0).as_str(),
            t.center() + Point::new(0, 12),
            text_style,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        Text::with_alignment(
            format!("Illuminance: {:.2?}", illuminance.expect("")).as_str(),
            t.center() + Point::new(0, 22),
            text_style,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        // Write buffer to display
        display.flush().unwrap();
        // Clear display buffer
        display.clear(BinaryColor::Off).unwrap();

        // Wait 5 seconds
        block!(timer0.wait()).unwrap();
    }
}
