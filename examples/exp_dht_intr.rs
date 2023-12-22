//! GPIO interrupt
//!
//! This prints "Interrupt" when the boot button is pressed.
//! It also blinks an LED like the blinky example.

#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;
use hal::{
    clock::ClockControl,
    gpio::{Event, Gpio2, Input, PullDown, IO},
    interrupt,
    peripherals::{self, Peripherals},
    prelude::*,
    riscv,
    Delay,
};
use esp_backtrace as _;
use esp_hal_common::gpio::{OpenDrain, Output};
use heapless::spsc::Queue;
use heapless::Vec;
use esp_hal_common::systimer::SystemTimer;

const QUEUE_SIZE: usize = 100;
static BUTTON: Mutex<RefCell<Option<Gpio2<Output<OpenDrain>>>>> = Mutex::new(RefCell::new(None));
static TIMESTAMP_QUEUE: Mutex<RefCell<Queue<u64, QUEUE_SIZE>>> = Mutex::new(RefCell::new(Queue::new()));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // Set GPIO5 as an output
    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    // Set GPIO9 as an input
    let mut button = io.pins.gpio2.into_open_drain_output();
    button.listen(Event::AnyEdge);

    critical_section::with(|cs| BUTTON.borrow_ref_mut(cs).replace(button));

    interrupt::enable(peripherals::Interrupt::GPIO, interrupt::Priority::Priority15).unwrap();

    unsafe {
        riscv::interrupt::enable();
    }

    let mut delay = Delay::new(&clocks);
    let mut intervals : Vec<u64, QUEUE_SIZE> = Vec::new();
    loop {
        critical_section::with(|cs| {
            let mut btn = BUTTON.borrow(cs).take().unwrap();
            btn.set_low().ok();
            delay.delay_ms(18u8);
            btn.set_high().ok();
            delay.delay_us(48u8);
            BUTTON.borrow_ref_mut(cs).replace(btn)
        });

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);

        critical_section::with(|cs| {
            let mut queue = TIMESTAMP_QUEUE.borrow_ref_mut(cs);
            intervals.clear();
            let mut total = 0u64;
            let mut last_ts = 0u64;
            while let Some(ts) = queue.dequeue() {
                if last_ts > 0 {
                    let it = (ts - last_ts) / 16;
                    total += it;
                    intervals.push(it);
                }
                last_ts = ts;
            }
            esp_println::println!("Intervals: {:?}, total: {}", intervals, total);
        });
    }
}

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        TIMESTAMP_QUEUE.borrow_ref_mut(cs).enqueue(SystemTimer::now());
        BUTTON.borrow_ref_mut(cs).as_mut().unwrap().clear_interrupt();
    });
}
