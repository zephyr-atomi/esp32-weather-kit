#![no_std]
#![no_main]

extern crate alloc;

use core::arch::asm;
use core::mem::MaybeUninit;
use esp_backtrace as _;
use esp_hal_common::systimer::SystemTimer;
use hal::{clock::ClockControl, peripherals::Peripherals, prelude::*, Delay, IO};

use esp_hal_common::timer::TimerGroup;
use heapless::spsc::Queue;
use log::info;

#[global_allocator]
static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();

fn init_heap() {
    const HEAP_SIZE: usize = 32 * 1024;
    static mut HEAP: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();

    unsafe {
        ALLOCATOR.init(HEAP.as_mut_ptr() as *mut u8, HEAP_SIZE);
    }
}

const TIMEOUT_TICKS: u64 = 160_000_0;  // 100ms

#[entry]
fn main() -> ! {
    init_heap();
    let peripherals = Peripherals::take();
    let system = peripherals.SYSTEM.split();

    let clocks = ClockControl::max(system.clock_control).freeze();
    let mut delay = Delay::new(&clocks);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = io.pins.gpio8.into_push_pull_output();

    let sys_timer = SystemTimer::new(peripherals.SYSTIMER);
    info!("SYSTIMER Current value = {}", SystemTimer::now());

    let mut dht11_pin = io.pins.gpio2.into_open_drain_output();

    esp_println::logger::init_logger_from_env();
    log::info!("Hello world!");
    let mut count = 0u32;

    loop {
        info!("Loop... {}, time: {}", count, SystemTimer::now());
        count += 1;
        // loop to read all data.
        let mut time_intervals = Queue::<u64, 100>::new();
        let mut last_time = SystemTimer::now();
        let mut last_state = false;

        dht11_pin.set_low().ok();
        delay.delay_ms(18_u8);
        dht11_pin.set_high().ok();
        delay.delay_us(48_u8);
        loop {
            // 检查 GPIO2 状态
            let current_state = dht11_pin.is_high().unwrap();

            let current_time = SystemTimer::now();
            let interval = current_time - last_time;
            if interval > TIMEOUT_TICKS {
                break
            }
            if current_state != last_state {
                last_time = current_time;
                last_state = current_state;
                time_intervals.enqueue(interval);
            }
        }

        info!("intervals: {:?}", time_intervals);

        led.set_high().unwrap();
        delay.delay_ms(500u32);

        led.set_low().unwrap();
        delay.delay_ms(500u32);
    }
}
