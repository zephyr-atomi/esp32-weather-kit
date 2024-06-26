//! This example sends a CAN message to another ESP and receives it back.
//!
//! Wiring:
//! This example works without CAN Transceivers by:
//! * setting the tx pins to open drain
//! * connecting all rx and tx pins together
//! * adding a pull-up to the signal pins
//!
//! ESP1/GND --- ESP2/GND
//! ESP1/IO0 --- ESP1/IO1 --- ESP2/IO0 --- ESP2/IO1 --- 4.8kOhm --- ESP1/5V
//!
//! `IS_FIRST_SENDER` below must be set to false on one of the ESP's

//% CHIPS: esp32c3 esp32s3
//% FEATURES: embedded-hal

#![no_std]
#![no_main]

const IS_FIRST_SENDER: bool = true;

use embedded_hal::can::{Can, Frame, StandardId};
// Run this example with the embedded-hal feature enabled to use embedded-can instead of
// embedded-hal-0.2.7. embedded-can was split off from embedded-hal before it's
// upgrade to 1.0.0. cargo run --example twai --release
#[cfg(feature = "embedded-hal")]
use embedded_can::{nb::Can, Frame, StandardId};
// Run this example without the embedded-hal flag to use the embedded-hal 0.2.7 CAN traits.
// cargo run --example twai --features embedded-hal-02 --release
#[cfg(feature = "embedded-hal-02")]
use embedded_hal_02::can::{Can, Frame, StandardId};
use esp_backtrace as _;
use esp_hal_common::clock::ClockControl;
use esp_hal_common::{IO, twai};
use esp_hal_common::peripherals::Peripherals;
use esp_hal_common::prelude::_esp_hal_system_SystemExt;
use esp_hal_common::prelude::nb::block;
use esp_hal_common::twai::filter::{Filter, SingleStandardFilter};
use esp_println::logger::init_logger;
use esp_println::println;
use hal::entry;
use log::{info, LevelFilter};

#[entry]
fn main() -> ! {
    init_logger(LevelFilter::Info);

    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    // Set the tx pin as open drain. Skip this if using transceivers.
    let can_tx_pin = io.pins.gpio0.into_open_drain_output();
    let can_rx_pin = io.pins.gpio1.into_floating_input();

    // The speed of the CAN bus.
    const CAN_BAUDRATE: twai::BaudRate = twai::BaudRate::B1000K;

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
        SingleStandardFilter::new(b"00000000000", b"x", [b"xxxxxxxx", b"xxxxxxxx"]);
    can_config.set_filter(FILTER);

    // let regs = FILTER.to_registers();
    // info!("Registers = {:x?}", regs);

    // Start the peripheral. This locks the configuration settings of the peripheral
    // and puts it into operation mode, allowing packets to be sent and
    // received.
    let mut can = can_config.start();
    //
    // if IS_FIRST_SENDER {
    //     // Send a frame to the other ESP
    //     let frame = Frame::new(StandardId::ZERO, &[1, 2, 3]).unwrap();
    //     block!(can.transmit(&frame)).unwrap();
    //     println!("Sent a frame");
    // }

    // Wait for a frame to be received.
    info!("Waiting for a can frame...");
    let frame = block!(can.receive()).unwrap();

    info!("Received a frame: {frame:?}");
    //
    // if !IS_FIRST_SENDER {
    //     // Transmit the frame back to the other ESP
    //     block!(can.transmit(&frame)).unwrap();
    //     println!("Sent a frame");
    // }

    loop {}
}
