use alloc::rc::Rc;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};
use core::cell::RefCell;

pub struct SharedI2cBus<I2C> {
    i2c_bus: Rc<RefCell<I2C>>,
}

impl<I2C> SharedI2cBus<I2C> {
    pub fn new(i2c_bus: I2C) -> Self {
        SharedI2cBus {
            i2c_bus: Rc::new(RefCell::new(i2c_bus)),
        }
    }
}

impl<I2C> Clone for SharedI2cBus<I2C> {
    fn clone(&self) -> Self {
        SharedI2cBus {
            i2c_bus: self.i2c_bus.clone(),
        }
    }
}

impl<I2C, E> Write for SharedI2cBus<I2C>
    where
        I2C: Write<Error = E>,
{
    type Error = E;

    fn write(&mut self, addr: u8, bytes: &[u8]) -> Result<(), Self::Error> {
        self.i2c_bus.borrow_mut().write(addr, bytes)
    }
}

impl<I2C, E> Read for SharedI2cBus<I2C>
    where
        I2C: Read<Error = E>,
{
    type Error = E;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c_bus.borrow_mut().read(addr, buffer)
    }
}

impl<I2C, E> WriteRead for SharedI2cBus<I2C>
    where
        I2C: WriteRead<Error = E>,
{
    type Error = E;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.i2c_bus.borrow_mut().write_read(addr, bytes, buffer)
    }
}
