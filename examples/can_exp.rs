#![no_std]
#![no_main]

use core::any::Any;
use embedded_hal::can::{Can, Frame, StandardId};
use hal::{clock::ClockControl, gpio::IO, i2c::I2C, peripherals::Peripherals, prelude::*, Delay};
use esp_backtrace as _;
use esp_hal_common::gpio::{GpioPin, Output};
use log::{error, info, LevelFilter};
use hal_exp::bh1750::{BH1750, MeasurementTime, Resolution};
use stepper_driver::MotorDriver;
use esp_hal_common::{gpio, twai};
use esp_hal_common::prelude::nb::block;
use esp_hal_common::twai::filter::{Filter, SingleStandardFilter};
use esp_hal_common::twai::{BaudRate, TimingConfig};
use esp_println::logger::init_logger;
use esp_println::println;

#[entry]
fn main() -> ! {
    init_logger(LevelFilter::Info);
    println!("Program started!");

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Set the tx pin as open drain. Skip this if using transceivers.
    let can_tx_pin = io.pins.gpio5;
    let can_rx_pin = io.pins.gpio4;

    // The speed of the CAN bus. (10Kbps)
    const CAN_BAUDRATE: twai::BaudRate = twai::BaudRate::Custom(TimingConfig {
        baud_rate_prescaler: 400,
        sync_jump_width: 3,
        tseg_1: 15,
        tseg_2: 4,
        triple_sample: false,
    });


    // IMPORTANT: Please don't use GPIO 1 for RX, due to issue here:
    // https://github.com/esp-rs/esp-hal/issues/1207

    // Begin configuring the TWAI peripheral. The peripheral is in a reset like
    // state that prevents transmission but allows configuration.
    let mut can_config = twai::TwaiConfiguration::new(
        peripherals.TWAI0,
        can_tx_pin,
        can_rx_pin,
        &clocks,
        CAN_BAUDRATE,
    );

    // Partially filter the incoming messages to reduce overhead of receiving
    // undesired messages. Note that due to how the hardware filters messages,
    // standard ids and extended ids may both match a filter. Frame ids should
    // be explicitly checked in the application instead of fully relying on
    // these partial acceptance filters to exactly match.
    // A filter that matches StandardId::ZERO.
    const FILTER: SingleStandardFilter =
        SingleStandardFilter::new(b"xxxxxxxxxxx", b"x", [b"xxxxxxxx", b"xxxxxxxx"]);

    let regs = FILTER.to_registers();
    info!("filter: {:x?}", regs);
    can_config.set_filter(FILTER);
    let mut can = can_config.start();

    let mut delay = Delay::new(&clocks);
    let mut led = io.pins.gpio8.into_push_pull_output();


    let mut count = 0;
    let mut out = 0u8;
    loop {
        count += 1;
        if count % 10_000_000 == 13 {
            // Send a frame
            out = out.wrapping_add(1);
            let frame = Frame::new(StandardId::new(0).unwrap(), &[0xaa, 0xbb, 0xcc, out]).unwrap();
            info!("Sent a frame, frame: {:?}", frame);
            let res = can.transmit(&frame);
            info!("Res: {:?}", res);

            led.toggle().unwrap();
        }
    }
}
