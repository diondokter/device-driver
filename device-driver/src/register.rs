use core::marker::PhantomData;

use crate::{FieldSet, ReadCapability, WriteCapability};

/// A trait to represent the interface to the device.
///
/// This is called to write to and read from registers.
pub trait RegisterInterface {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

    /// Write the given data to the register located at the given address
    fn write_register(
        &mut self,
        address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Read the register located at the given addres to the given data slice
    fn read_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to asynchronously write to and read from registers.
pub trait AsyncRegisterInterface {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

    /// Write the given data to the register located at the given address
    async fn write_register(
        &mut self,
        address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Read the register located at the given addres to the given data slice
    async fn read_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// Object that performs actions on the device in the context of a register
pub struct RegisterOperation<'i, Interface, AddressType: Copy, Register: FieldSet, Access> {
    interface: &'i mut Interface,
    address: AddressType,
    _phantom: PhantomData<(Register, Access)>,
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
{
    #[doc(hidden)]
    pub fn new(interface: &'i mut Interface, address: AddressType) -> Self {
        Self {
            interface,
            address,
            _phantom: PhantomData,
        }
    }
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub fn write<R>(&mut self, f: impl FnOnce(&mut Register) -> R) -> Result<R, Interface::Error> {
        let mut register = Register::new_with_default();
        let returned = f(&mut register);

        let buffer = Register::BUFFER::from(register);
        self.interface
            .write_register(self.address, buffer.as_ref())?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero<R>(
        &mut self,
        f: impl FnOnce(&mut Register) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = Register::new_with_zero();
        let returned = f(&mut register);
        self.interface
            .write_register(self.address, Register::BUFFER::from(register).as_mut())?;
        Ok(returned)
    }
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read the register from the device
    pub fn read(&mut self) -> Result<Register, Interface::Error> {
        let mut buffer = Register::BUFFER::from(Register::new_with_zero());

        self.interface
            .read_register(self.address, buffer.as_mut())?;
        Ok(buffer.into())
    }
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub fn modify<R>(&mut self, f: impl FnOnce(&mut Register) -> R) -> Result<R, Interface::Error> {
        let mut register = self.read()?;
        let returned = f(&mut register);
        self.interface
            .write_register(self.address, Register::BUFFER::from(register).as_mut())?;
        Ok(returned)
    }
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
where
    Interface: AsyncRegisterInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub async fn write_async<R>(
        &mut self,
        f: impl FnOnce(&mut Register) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = Register::new_with_default();
        let returned = f(&mut register);

        let buffer = Register::BUFFER::from(register);
        self.interface
            .write_register(self.address, buffer.as_ref())
            .await?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub async fn write_with_zero_async<R>(
        &mut self,
        f: impl FnOnce(&mut Register) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = Register::new_with_zero();
        let returned = f(&mut register);
        self.interface
            .write_register(self.address, Register::BUFFER::from(register).as_mut())
            .await?;
        Ok(returned)
    }
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
where
    Interface: AsyncRegisterInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read the register from the device
    pub async fn read_async(&mut self) -> Result<Register, Interface::Error> {
        let mut buffer = Register::BUFFER::from(Register::new_with_zero());

        self.interface
            .read_register(self.address, buffer.as_mut())
            .await?;
        Ok(buffer.into())
    }
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
where
    Interface: AsyncRegisterInterface<AddressType = AddressType>,
    Access: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub async fn modify_async<R>(
        &mut self,
        f: impl FnOnce(&mut Register) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = self.read_async().await?;
        let returned = f(&mut register);
        self.interface
            .write_register(self.address, Register::BUFFER::from(register).as_mut())
            .await?;
        Ok(returned)
    }
}
