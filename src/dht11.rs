use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use log::info;


const MAX_COUNT: u16 = 200;

#[derive(Debug)]
pub enum Error<E> {
    /// Timeout during communication.
    Timeout,
    /// CRC mismatch.
    CrcMismatch,
    IncorrectBits,
    /// GPIO error.
    Gpio(E),
}

pub fn read_and_decode_dht11<GPIO, D, E>(dht11_pin: &mut GPIO, delay: &mut D) -> Result<[u8; 5], Error<E>>
    where
        GPIO: InputPin<Error = E> + OutputPin<Error = E>,
        D: DelayUs<u8> + DelayMs<u8>,
{
    let mut bit_data = [0u8; 40]; // 40 bits to read
    let mut bit_index = 0;

    let mut c0 = [0u16; 40];
    let mut c1 = [0u16; 40];

    dht11_pin.set_low().ok();
    delay.delay_ms(18_u8);
    dht11_pin.set_high().ok();
    delay.delay_us(100u8);

    // Skip the initial sequence of 1s
    while dht11_pin.is_high().unwrap_or(false) {
        delay.delay_us(8u8);
    }
    // Read 40 bits
    while bit_index < 40 {
        // Count consecutive 0s and 1s
        let mut count_0 = 0u16;
        let mut count_1 = 0u16;

        // Count 0s
        while dht11_pin.is_low().unwrap_or(false) {
            count_0 += 1;
            if count_0 > MAX_COUNT {
                break;
            }
            delay.delay_us(8u8);
        }

        // Count 1s
        while dht11_pin.is_high().unwrap_or(false) {
            count_1 += 1;
            if count_1 > MAX_COUNT {
                break;
            }
            delay.delay_us(8u8);
        }

        if count_0 > MAX_COUNT || count_1 > MAX_COUNT {
            break;
        }

        // Determine bit value based on counts
        bit_data[bit_index] = if count_1 > count_0 { 1 } else { 0 };
        c0[bit_index] = count_0;
        c1[bit_index] = count_1;

        bit_index += 1;
    }

    info!("bit_index = {}", bit_index);
    info!("bit_data: {:?}", bit_data);
    info!("c0: {:?}", c0);
    info!("c1: {:?}", c1);

    // Decode the read bits into 5 bytes
    let mut result = [0u8; 5];
    let mut current_byte = 0;
    let mut bit_position = 7;

    for bit in bit_data.iter() {
        if *bit == 1 {
            result[current_byte] |= 1 << bit_position;
        }
        bit_position -= 1;

        if bit_position < 0 {
            bit_position = 7;
            current_byte += 1;
        }
    }

    if current_byte != 5 {
        return Err(Error::IncorrectBits);
    }

    Ok(result)
}
