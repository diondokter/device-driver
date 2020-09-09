use device_driver::ll::register::RegisterInterface;
use device_driver::{create_low_level_device, implement_registers, ll::register::RegisterError};

use device_driver::ll::LowLevelError;
use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_mock::{pin, spi};
use std::fmt::Debug;

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

impl<SPI: Transfer<u8> + Write<u8>, CS: OutputPin, RESET: OutputPin> ChipInterface<SPI, CS, RESET> {
    pub fn free(self) -> (SPI, CS, RESET) {
        (self.communication_interface, self.cs_pin, self.reset_pin)
    }
}

// Implementing the register interface for the hardware interface
impl<SPI: Transfer<u8> + Write<u8>, CS: OutputPin, RESET: OutputPin> RegisterInterface
    for ChipInterface<SPI, CS, RESET>
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
implement_registers!(
    /// The global register set
    MyDevice.registers<u8> = {
        /// The identification register
        id(RO, 0, 2) = {
            /// The manufacturer code
            manufacturer: u8 = RW 0..8,
            /// The version of the chip
            version: u8 = RO 8..16,
        },
        port(WO, 1, 1) = {
            output_0: u8 = WO 0..=0,
            output_1: u8 = WO 1..=1,
            output_2: u8 = WO 2..=2,
            output_3: u8 = WO 3..=3,
            mask_0: u8 = WO 4..=4,
            mask_1: u8 = WO 5..=5,
            mask_2: u8 = WO 6..=6,
            mask_3: u8 = WO 7..=7,
        },
        pin(RO, 2, 1) = {
            input_0: u8 = RO 0..=1,
            input_1: u8 = RO 2..=3,
            input_2: u8 = RO 4..=5,
            input_3: u8 = RO 6..=7,
        },
        mode(RW, 2, 1) = {
            mode_0: u8 = RW 0..=1,
            mode_1: u8 = RW 2..=3,
            mode_2: u8 = RW 4..=5,
            mode_3: u8 = RW 6..=7,
        },
    }
);

fn main() {
    let spi_expectations = [
        // Read ID register
        spi::Transaction::write(vec![0x80]),
        spi::Transaction::transfer(vec![0x00, 0x00], vec![0xDE, 0xAD]),
        // Read Mode register
        spi::Transaction::write(vec![0x82]),
        spi::Transaction::transfer(vec![0x00], vec![0b11100100]),
        // Write Mode register
        spi::Transaction::write(vec![0x02]),
        spi::Transaction::write(vec![0b11100101]),
        // Write Port register
        spi::Transaction::write(vec![0x01]),
        spi::Transaction::write(vec![0x11]),
    ];

    let cs_expectations = [
        pin::Transaction::set(pin::State::Low),
        pin::Transaction::set(pin::State::High),
        pin::Transaction::set(pin::State::Low),
        pin::Transaction::set(pin::State::High),
        pin::Transaction::set(pin::State::Low),
        pin::Transaction::set(pin::State::High),
        pin::Transaction::set(pin::State::Low),
        pin::Transaction::set(pin::State::High),
    ];

    let reset_expectations = [];

    let mut device = MyDevice::new(ChipInterface {
        communication_interface: spi::Mock::new(&spi_expectations),
        cs_pin: pin::Mock::new(&cs_expectations),
        reset_pin: pin::Mock::new(&reset_expectations),
    });

    example(&mut device).unwrap();

    let (mut spi, mut cs, mut reset) = device.free().free();

    spi.done();
    cs.done();
    reset.done();
}

fn example<SPI, CS, RESET>(
    device: &mut MyDevice<ChipInterface<SPI, CS, RESET>>,
) -> Result<(), LowLevelError<InterfaceError>>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
    RESET: OutputPin,
{
    let manufacturer = device.registers().id().read()?.manufacturer();

    if manufacturer != 0 {
        device.registers().mode().modify(|_, w| w.mode_0(1))?;
        device
            .registers()
            .port()
            .write(|w| w.output_0(1).mask_0(1))?;
    }

    Ok(())
}
