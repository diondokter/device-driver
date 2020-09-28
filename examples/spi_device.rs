use device_driver::ll::register::RegisterInterface;
use device_driver::{create_low_level_device, implement_registers, Bit};

use embedded_hal::blocking::spi::{Transfer, Write};
use embedded_hal::digital::v2::OutputPin;
use embedded_hal_mock::{pin, spi};
use num_enum::{IntoPrimitive, TryFromPrimitive};
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

/// Mark this interface so it can be used
impl<SPI: Transfer<u8> + Write<u8>, CS: OutputPin, RESET: OutputPin> HardwareInterface
    for ChipInterface<SPI, CS, RESET>
{
    fn reset(&mut self) -> Result<(), InterfaceError> {
        self.reset_pin
            .set_high()
            .map_err(|_| InterfaceError::ResetError)?;
        self.reset_pin
            .set_low()
            .map_err(|_| InterfaceError::ResetError)?;

        Ok(())
    }
}

// Create our low level device. This holds all the hardware communication definitions
create_low_level_device!(
    /// Our test device
    MyDevice {
        // The types of errors our low level error enum must contain
        errors: [InterfaceError],
        hardware_interface_requirements: { RegisterInterface },
        hardware_interface_capabilities: {
            fn reset(&mut self) -> Result<(), InterfaceError>;
        },
    }
);

// Create a register set for the device
implement_registers!(
    /// The global register set
    MyDevice.registers<u8> = {
        /// The identification register
        id(RO, 0, 2) = {
            /// The manufacturer code
            manufacturer: u8 as Manufacturer = RW 0..8,
            /// The version of the chip
            version: u8 = RO 8..16,
        },
        /// The output register.
        ///
        /// The output value will only be updated for the output of which the mask bit is also set.
        port(WO, 1, 1) = {
            /// Sets output 0 if mask 0 is also high
            output_0: u8 as Bit = WO 0..=0,
            /// Sets output 1 if mask 0 is also high
            output_1: u8 as Bit = WO 1..=1,
            /// Sets output 2 if mask 0 is also high
            output_2: u8 as Bit = WO 2..=2,
            /// Sets output 3 if mask 0 is also high
            output_3: u8 as Bit = WO 3..=3,
            /// Mask bit for output 0
            mask_0: u8 as Bit = WO 4..=4,
            /// Mask bit for output 1
            mask_1: u8 as Bit = WO 5..=5,
            /// Mask bit for output 2
            mask_2: u8 as Bit = WO 6..=6,
            /// Mask bit for output 3
            mask_3: u8 as Bit = WO 7..=7,
        },
        /// The input register
        pin(RO, 2, 1) = {
            /// The input state for pin 0
            input_0: u8 as PinInputState = RO 0..=1,
            /// The input state for pin 1
            input_1: u8 as PinInputState = RO 2..=3,
            /// The input state for pin 2
            input_2: u8 as PinInputState = RO 4..=5,
            /// The input state for pin 3
            input_3: u8 as PinInputState = RO 6..=7,
        },
        /// The pin mode register
        mode(RW, [3, 4, 5, 6], 1) = {
            /// The mode of the pin
            mode: u8 as PinMode = RW 0..=1,
        },
    }
);

/// The mode of the pin. 2 bits.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum PinMode {
    InputFloating = 0b00,
    InputPullup = 0b01,
    InputPulldown = 0b10,
    Output = 0b11,
}

/// The state of the input pin. 2 bits.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum PinInputState {
    Floating = 0b00,
    High = 0b01,
    Low = 0b10,
}

/// The name of the manufacturer. 8 bits.
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum Manufacturer {
    Unknown = 0x00,
    CarmineCrystal = 0xCC,
}

fn main() {
    let spi_expectations = [
        // Read ID register
        spi::Transaction::write(vec![0x80]),
        spi::Transaction::transfer(vec![0x00, 0x00], vec![0xCC, 0x05]),
        // Read Mode register
        spi::Transaction::write(vec![0x83]),
        spi::Transaction::transfer(vec![0x00], vec![0b11100100]),
        // Write Mode register
        spi::Transaction::write(vec![0x03]),
        spi::Transaction::write(vec![0b11100111]),
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

    let reset_expectations = [
        pin::Transaction::set(pin::State::High),
        pin::Transaction::set(pin::State::Low),
    ];

    let mut device = MyDevice::new(ChipInterface {
        communication_interface: spi::Mock::new(&spi_expectations),
        cs_pin: pin::Mock::new(&cs_expectations),
        reset_pin: pin::Mock::new(&reset_expectations),
    });

    device.interface().reset().unwrap();

    run(&mut device).unwrap();

    let (mut spi, mut cs, mut reset) = device.free().free();

    spi.done();
    cs.done();
    reset.done();
}

fn run<SPI, CS, RESET>(
    device: &mut MyDevice<ChipInterface<SPI, CS, RESET>>,
) -> Result<(), LowLevelError>
where
    SPI: Transfer<u8> + Write<u8>,
    CS: OutputPin,
    RESET: OutputPin,
{
    let manufacturer = device.registers().id().read()?.manufacturer()?;

    if manufacturer != Manufacturer::Unknown {
        device
            .registers()
            .mode()
            .modify_index(0, |_, w| w.mode(PinMode::Output))?;
        device
            .registers()
            .port()
            .write(|w| w.output_0(Bit::Set).mask_0(Bit::Set))?;
    }

    Ok(())
}
