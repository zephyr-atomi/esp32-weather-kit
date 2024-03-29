//! This shows how to use the SYSTIMER peripheral including interrupts.
//! It's an additional timer besides the TIMG peripherals.

#![no_std]
#![no_main]

use core::cell::RefCell;

use critical_section::Mutex;
use hal::{
    clock::ClockControl,
    interrupt,
    interrupt::Priority,
    peripherals::{self, Peripherals},
    prelude::*,
    systimer::{Alarm, Periodic, SystemTimer, Target},
    Delay,
};
use esp_backtrace as _;
use esp_println::println;

static ALARM0: Mutex<RefCell<Option<Alarm<Periodic, 0>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let syst = SystemTimer::new(peripherals.SYSTIMER);

    println!("SYSTIMER Current value = {}", SystemTimer::now());

    let alarm0 = syst.alarm0.into_periodic();
    alarm0.set_period(10u32.secs());
    alarm0.clear_interrupt();
    alarm0.enable_interrupt(true);

    critical_section::with(|cs| {
        ALARM0.borrow_ref_mut(cs).replace(alarm0);
    });

    interrupt::enable(
        peripherals::Interrupt::SYSTIMER_TARGET0,
        Priority::Priority1,
    )
        .unwrap();

    // Initialize the Delay peripheral, and use it to toggle the LED state in a
    // loop.
    let mut delay = Delay::new(&clocks);

    loop {
        delay.delay_ms(500u32);
    }
}

#[interrupt]
fn SYSTIMER_TARGET0() {
    println!("Interrupt lvl1 (alarm0). {}", SystemTimer::now() / 16);
    critical_section::with(|cs| {
        ALARM0
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt()
    });
}
