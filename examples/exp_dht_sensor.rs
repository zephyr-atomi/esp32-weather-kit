#![no_std]
#![no_main]

extern crate alloc;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO};
use dht_sensor::dht11;
use dht_sensor::DhtReading;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}
#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    let mut dht11_pin = io.pins.gpio2.into_open_drain_output();

    esp_println::logger::init_logger_from_env();
    log::info!("Hello world!");
    let mut count = 0u32;

    loop {
        log::info!("Loop... {}", count);
        count += 1;

        match dht11::Reading::read(&mut delay, &mut dht11_pin) {
            Ok(reading) => {
                log::info!("readings: {:?}", reading)
            }
            Err(e) => {
                log::error!("DHT11 reading error: {:?}", e)
            }
        }

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
