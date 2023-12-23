use alloc::string::String;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use log::info;

fn decode_dht11_signal(res: &[u8]) -> [u8; 5] {
    let mut result = [0u8; 5];
    let mut current_byte = 0;
    let mut bit_index = 7;

    // Skip initial sequence of 1s
    let mut index = 0;
    while index < res.len() && res[index] == 1 {
        index += 1;
    }

    while index < res.len() && current_byte < 5 {
        // Count consecutive 0s and 1s
        let mut count_0 = 0;
        let mut count_1 = 0;

        // Count 0s
        while index < res.len() && res[index] == 0 {
            count_0 += 1;
            index += 1;
        }

        // Count 1s
        while index < res.len() && res[index] == 1 {
            count_1 += 1;
            index += 1;
        }

        // Determine bit value based on counts
        if count_1 > count_0 {
            result[current_byte] |= 1 << bit_index;
        }

        // Move to next bit
        bit_index -= 1;

        // Move to next byte if current one is complete
        if bit_index < 0 {
            current_byte += 1;
            bit_index = 7;
        }
    }

    info!("Current index pos: {}", index);

    result
}

fn ref_dht11<GPIO, D, E>(dht11_pin: &mut GPIO, ets_delay: &mut D) -> Result<[u8; 5], E>
    where
        GPIO: InputPin<Error = E> + OutputPin<Error = E>,
        D: DelayUs<u8> + DelayMs<u8>
{
    let mut res = [0; 500];

    dht11_pin.set_low().ok();
    ets_delay.delay_ms(18_u8);
    dht11_pin.set_high().ok();
    ets_delay.delay_us(100u8);

    for i in 0..500 {
        ets_delay.delay_us(8u8);
        res[i] = if dht11_pin.is_high()? {1} else {0};
    }
    let res_str: String = res.iter().map(|&bit| if bit == 1 { '1' } else { '0' }).collect();
    info!("Data sequence: {}", res_str);

    Ok(decode_dht11_signal(&res))
}
