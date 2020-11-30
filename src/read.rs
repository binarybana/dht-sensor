use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};

#[derive(Debug)]
pub enum DhtError<E> {
    PinError(E),
    ChecksumMismatch,
}

impl<E> From<E> for DhtError<E> {
    fn from(error: E) -> DhtError<E> {
        DhtError::PinError(error)
    }
}

pub trait Delay: DelayUs<u8> + DelayMs<u8> {}
impl<T> Delay for T where T: DelayMs<u8> + DelayUs<u8> {}

pub trait InputOutputPin<E>: InputPin<Error = E> + OutputPin<Error = E> {}
impl<T, E> InputOutputPin<E> for T where T: InputPin<Error = E> + OutputPin<Error = E> {}

fn read_bit<D, E>(delay: &mut D, pin: &impl InputPin<Error = E>) -> Result<bool, E>
where
    D: DelayUs<u8>,
{
    while pin.is_low()? {}
    delay.delay_us(35u8);
    // delay.delay_us(30u8);
    // delay.delay_us(100u8);
    let high = pin.is_high()?;
    while pin.is_high()? {}
    Ok(high)
}

fn read_byte<D, E>(delay: &mut D, pin: &impl InputPin<Error = E>) -> Result<u8, E>
where
    D: DelayUs<u8>,
{
    let mut byte: u8 = 0;
    for i in 0..8 {
        let bit_mask = 1 << (7 - (i % 8));
        if read_bit(delay, pin)? {
            byte |= bit_mask;
        }
    }
    Ok(byte)
}

pub fn read_raw<P, E, D>(delay: &mut D, pin: &mut P) -> Result<[u8; 4], DhtError<E>>
where
    P: InputOutputPin<E>,
    D: Delay,
{
    use std::io::{self, Write};
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    handle.write_all(b"hello world").unwrap();
    pin.set_low().ok();
    handle.write_all(b"Pin has been set low").unwrap();
    handle.flush().unwrap();
    // delay.delay_ms(18_u8);
    delay.delay_ms(30_u8);
    pin.set_high().ok();
    handle.write_all(b"Pin has been set high").unwrap();
    handle.flush().unwrap();
    // delay.delay_us(48_u8);
    delay.delay_us(40_u8);
    while pin.is_low()? {}
    handle.write_all(b"Pin is no longer low").unwrap();
    handle.flush().unwrap();
    while pin.is_high()? {}
    handle
        .write_all(b"Pin is no longer high, ready to start reading")
        .unwrap();
    handle.flush().unwrap();
    let mut data = [0; 4];
    for b in data.iter_mut() {
        *b = read_byte(delay, pin)?;
        handle.write_all(b"read byte").unwrap();
        handle.flush().unwrap();
    }
    let checksum = read_byte(delay, pin)?;
    if data.iter().fold(0u8, |acc, x| acc.wrapping_add(*x)) != checksum {
        Err(DhtError::ChecksumMismatch)
    } else {
        Ok(data)
    }
}
