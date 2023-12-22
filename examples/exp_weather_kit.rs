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
use log::info;
use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};
use hal_exp::bmp180::{BMP180, Oversampling};
use alloc::format;
use hal_exp::shared_i2c::SharedI2cBus;

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

    // Start timer (5 second interval)
    timer0.start(5u64.secs());
    loop {
        bh1750.set_resolution(Resolution::Lx1_0);
        let illuminance = bh1750.illuminance();
        let (temp, pressure) = bmp180.temperature_and_pressure(Oversampling::O1).unwrap();

        log::info!("Temp: {:.2}C Pressure: {:.2}hPa", temp as f32 / 10.0, pressure as f32 / 100.0);

        // Fill display bufffer with a centered text with two lines (and two text
        // styles)
        let t = display.bounding_box();
        info!("box: {:?}", t);
        Text::with_alignment(
            format!("Temp: {:.2}C", temp as f32 / 10.0).as_str(),
            t.center() + Point::new(0, -5),
            text_style_big,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        Text::with_alignment(
            format!("Pressure: {:.2}hPa", pressure as f32 / 100.0).as_str(),
            t.center() + Point::new(0, 10),
            text_style,
            Alignment::Center,
        )
            .draw(&mut display)
            .unwrap();

        Text::with_alignment(
            format!("Illuminance: {:.2?}", illuminance.expect("")).as_str(),
            t.center() + Point::new(0, 20),
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
