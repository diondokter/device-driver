use device_driver::ll::register::RegisterInterface;
use device_driver::{create_low_level_device, implement_registers, ll::register::RegisterError};

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;

/// Mock impl for hal spi
pub struct MockSpi;
impl Transfer<u8> for MockSpi {
    type Error = ();
    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Self::Error> {
        println!("Spi transferred {:x?}", words);
        Ok(words)
    }
}
impl Write<u8> for MockSpi {
    type Error = ();
    fn write(&mut self, words: &[u8]) -> Result<(), Self::Error> {
        println!("Spi wrote {:x?}", words);
        Ok(())
    }
}

// Mock impl for output pin
pub struct MockPin;
impl OutputPin for MockPin {
    type Error = ();
    fn set_low(&mut self) -> Result<(), Self::Error> {
        println!("Pin set low");
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        println!("Pin set high");
        Ok(())
    }
}

#[derive(Debug)]
pub enum InterfaceError {
    CsError,
    ResetError,
    CommunicationError,
}

/// Our full hardware interface with the chip
pub struct ChipInterface<SPI: Transfer<u8> + Write<u8>, CS: OutputPin, RESET: OutputPin> {
    pub communication_interface: SPI,
    pub cs_pin: CS,
    pub reset_pin: RESET,
}

// Implementing the register interface for the hardware interface
impl<SPI: Transfer<u8> + Write<u8>, CS: OutputPin, RESET: OutputPin> RegisterInterface
    for ChipInterface<SPI, RESET, CS>
{
    type Address = u8;
    type InterfaceError = InterfaceError;

    fn read_register(
        &mut self,
        address: Self::Address,
        value: &mut [u8],
    ) -> Result<(), Self::InterfaceError> {
        self.cs_pin
            .set_low()
            .map_err(|_| Self::InterfaceError::CsError)?;

        self.communication_interface
            .write(&[0x80 | address])
            .map_err(|_| Self::InterfaceError::CommunicationError)?;
        self.communication_interface
            .transfer(value)
            .map_err(|_| Self::InterfaceError::CommunicationError)?;

        self.cs_pin
            .set_high()
            .map_err(|_| Self::InterfaceError::CsError)?;
        Ok(())
    }

    fn write_register(
        &mut self,
        address: Self::Address,
        value: &[u8],
    ) -> Result<(), Self::InterfaceError> {
        self.cs_pin
            .set_low()
            .map_err(|_| Self::InterfaceError::CsError)?;

        self.communication_interface
            .write(&[address])
            .map_err(|_| Self::InterfaceError::CommunicationError)?;
        self.communication_interface
            .write(value)
            .map_err(|_| Self::InterfaceError::CommunicationError)?;

        self.cs_pin
            .set_high()
            .map_err(|_| Self::InterfaceError::CsError)?;

        Ok(())
    }
}

// Create our low level device. This holds all the hardware communication definitions
create_low_level_device!(MyDevice);

// Create a register set for the device
implement_registers!(MyDevice.registers<u8> = {
    id(RW, 0, 4) = {

    },
    test(RO, 1, 2) = {

    },
    test2(WO, 2, 1) = {

    }
});

fn main() {
    let mut device = MyDevice::new(ChipInterface {
        communication_interface: MockSpi,
        cs_pin: MockPin,
        reset_pin: MockPin,
    });

    device.registers().id().modify(|_, w| w).unwrap();
}
