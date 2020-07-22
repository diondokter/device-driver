#![feature(trait_alias)]

use device_driver::{create_device, implement_registers, ll::register::RegisterError};

use embedded_hal::blocking::spi::{Transfer, Write};

/// Mock impl for hal spi
pub struct Spi;
impl Transfer<u8> for Spi {
    type Error = ();
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        Ok(words)
    }
}
impl Write<u8> for Spi {
    type Error = ();
    fn write(&mut self, _words: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// A wrapper around a
pub struct InterfaceWrapper<E, SPI: Transfer<u8, Error = E> + Write<u8, Error = E>> {
    pub interface: SPI,
}

// Implementing the register interface for the wrapper
impl<E, SPI: Transfer<u8, Error = E> + Write<u8, Error = E>> RegisterInterface
    for InterfaceWrapper<E, SPI>
{
    type Word = u8;
    type Address = u8;
    type InterfaceError = E;

    fn read_register(
        &mut self,
        address: Self::Address,
    ) -> Result<Self::Word, RegisterError<Self::InterfaceError>> {
        Ok(self.interface.transfer(&mut [address, 0])?[1])
    }
    fn write_register(
        &mut self,
        _address: Self::Address,
        _value: Self::Word,
    ) -> Result<(), RegisterError<Self::InterfaceError>> {
        Ok(())
    }
}

pub struct ResetPin;
impl embedded_hal::digital::v2::OutputPin for ResetPin {
    type Error = ();
    fn set_low(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        todo!()
    }
}

create_device!(MyDevice {
    //interface: embedded_hal::digital::v2::OutputPin,
    //pins: embedded_hal::digital::v2::OutputPin,
    error: (),
});

implement_registers!(MyDevice { id, test });

fn main() {
    let mut device = MyDevice::new(InterfaceWrapper { interface: Spi }, ResetPin).unwrap();

    device.registers().id();
}
