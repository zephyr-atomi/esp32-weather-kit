use embedded_hal::blocking::delay::{DelayMs, DelayUs};

pub struct EtsDelay;

extern "C" {
    fn ets_delay_us(us: u32);
}

impl EtsDelay {
    pub fn delay_us(us: u32) {
        unsafe {
            ets_delay_us(us);
        }
    }

    pub fn delay_ms(ms: u32) {
        unsafe {
            ets_delay_us(ms * 1000);
        }
    }
}

impl DelayUs<u16> for EtsDelay {
    fn delay_us(&mut self, us: u16) {
        EtsDelay::delay_us(us as _);
    }
}

impl DelayMs<u16> for EtsDelay {
    fn delay_ms(&mut self, ms: u16) {
        EtsDelay::delay_ms(ms as _);
    }
}

impl DelayUs<u8> for EtsDelay {
    fn delay_us(&mut self, us: u8) {
        EtsDelay::delay_us(us as _);
    }
}

impl DelayMs<u8> for EtsDelay {
    fn delay_ms(&mut self, ms: u8) {
        EtsDelay::delay_ms(ms as _);
    }
}
