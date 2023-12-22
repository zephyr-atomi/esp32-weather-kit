//! Platform-agnostic Rust driver for the DHT11 temperature and humidity sensor,
//! using [`embedded-hal`](https://github.com/rust-embedded/embedded-hal) traits.
//!
//! # Examples
//!
//! An example for the STM32F407 microcontroller can be found in the source repository.
//!
//! ```
//! // Open drain pin compatibible with HAL
//! let pin = gpio.pe2.into_open_drain_output();
//!
//! // SysTick-based HAL `Delay` on Cortex-M
//! let mut delay = Delay::new(cp.SYST, clocks);
//!
//! let mut dht11 = Dht11::new(pin);
//!
//! match dht11.perform_measurement(&mut delay) {
//!     Ok(meas) => hprintln!("Temp: {} Hum: {}", meas.temperature, meas.humidity).unwrap(),
//!     Err(e) => hprintln!("Error: {:?}", e).unwrap(),
//! };
//! ```
//!
//! # Optional features
//!
//! ## `dwt`
//!
//! In applications with multiple, frequent interrupts being handled by the system, the traditional
//! delay-based approach achievable using only traits provided by `embedded-hal` may lead to
//! incorrect bit readings, and thus timeout and CRC errors.
//!
//! Enabling this feature will cause the crate to use the DWT counter available on some ARM Cortex-M
//! families to more accurately measure bit times, thus greatly reducing inaccuracies introduced
//! by external interrupts.

#![deny(unsafe_code)]
#![deny(missing_docs)]

use embedded_hal::{
    blocking::delay::{DelayMs, DelayUs},
    digital::v2::{InputPin, OutputPin},
};
use esp_hal_common::systimer::SystemTimer;
use log::info;

#[cfg(feature = "dwt")]
use cortex_m::peripheral::DWT;

/// How long to wait for a pulse on the data line (in microseconds).
const TIMEOUT_COUNT: u32 = 0xFFFFF;

/// Error type for this crate.
#[derive(Debug)]
pub enum Error<E> {
    /// Timeout during communication.
    Timeout,
    /// CRC mismatch.
    CrcMismatch,
    /// GPIO error.
    Gpio(E),
}

/// A DHT11 device.
pub struct Dht11<GPIO> {
    /// The concrete GPIO pin implementation.
    gpio: GPIO,
}

/// Results of a reading performed by the DHT11.
#[derive(Copy, Clone, Default, Debug)]
pub struct Measurement {
    /// The measured temperature in tenths of degrees Celsius.
    pub temperature: i16,
    /// The measured humidity in tenths of a percent.
    pub humidity: u16,
}

impl<GPIO, E> Dht11<GPIO>
    where
        GPIO: InputPin<Error=E> + OutputPin<Error=E>,
{
    /// Creates a new DHT11 device connected to the specified pin.
    pub fn new(gpio: GPIO) -> Self {
        Dht11 { gpio }
    }

    /// Destroys the driver, returning the GPIO instance.
    pub fn destroy(self) -> GPIO {
        self.gpio
    }

    /// Performs a reading of the sensor.
    pub fn perform_measurement<D>(&mut self, delay: &mut D) -> Result<Measurement, Error<E>>
        where
            D: DelayUs<u16> + DelayMs<u16>,
    {
        let mut data = [0u8; 5];

        // Perform initial handshake
        self.perform_handshake(delay)?;

        // Read bits
        for i in 0..40 {
            data[i / 8] <<= 1;
            if self.read_bit(delay)? {
                data[i / 8] |= 1;
            }
        }

        // Finally wait for line to go idle again.
        self.wait_for_pulse(true, delay)?;

        // Check CRC
        let crc = data[0]
            .wrapping_add(data[1])
            .wrapping_add(data[2])
            .wrapping_add(data[3]);
        if crc != data[4] {
            return Err(Error::CrcMismatch);
        }

        // Compute temperature
        let mut temp = i16::from(data[2] & 0x7f) * 10 + i16::from(data[3]);
        if data[2] & 0x80 != 0 {
            temp = -temp;
        }

        Ok(Measurement {
            temperature: temp,
            humidity: u16::from(data[0]) * 10 + u16::from(data[1]),
        })
    }

    fn perform_handshake<D>(&mut self, delay: &mut D) -> Result<(), Error<E>>
        where
            D: DelayUs<u16> + DelayMs<u16>,
    {
        // Set pin as floating to let pull-up raise the line and start the reading process.
        self.set_input()?;
        delay.delay_ms(1);

        // Pull line low for at least 18ms to send a start command.
        self.set_low()?;
        delay.delay_ms(20);

        // Restore floating
        self.set_input()?;
        delay.delay_us(40);

        // As a response, the device pulls the line low for 80us and then high for 80us.
        self.read_bit(delay)?;

        Ok(())
    }

    fn read_bit<D>(&mut self, delay: &mut D) -> Result<bool, Error<E>>
        where
            D: DelayUs<u16> + DelayMs<u16>,
    {
        let low = self.wait_for_pulse(true, delay)?;
        let high = self.wait_for_pulse(false, delay)?;
        // info!("low: {}, high: {}", low, high);
        Ok(high > low)
    }

    fn wait_for_pulse<D>(&mut self, level: bool, delay: &mut D) -> Result<u64, Error<E>>
        where
            D: DelayUs<u16> + DelayMs<u16>,
    {
        let mut count = 0u32;

        // let mut timer = Timer::new(TIMG0, ());
        let start_time = SystemTimer::now();

        #[cfg(feature = "dwt")]
            let start = DWT::get_cycle_count();

        while self.read_line()? != level {
            count += 1;
            if count > TIMEOUT_COUNT {
                return Err(Error::Timeout);
            }
            // delay.delay_us(1);
        }
        let delta = SystemTimer::now() - start_time;
        // info!("start_time: {}, end_time: {}, delta: {}", start_time, end_time, end_time - start_time);

        #[cfg(feature = "dwt")]
        return Ok(DWT::get_cycle_count().wrapping_sub(start));

        #[cfg(not(feature = "dwt"))]
        return Ok(delta);
    }

    fn set_input(&mut self) -> Result<(), Error<E>> {
        self.gpio.set_high().map_err(Error::Gpio)
    }

    fn set_low(&mut self) -> Result<(), Error<E>> {
        self.gpio.set_low().map_err(Error::Gpio)
    }

    fn read_line(&self) -> Result<bool, Error<E>> {
        self.gpio.is_high().map_err(Error::Gpio)
    }
}