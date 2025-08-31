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
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Read the register located at the given addres to the given data slice
    fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
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
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Read the register located at the given addres to the given data slice
    async fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// Object that performs actions on the device in the context of a register
pub struct RegisterOperation<'i, Interface, AddressType: Copy, Register: FieldSet, Access> {
    interface: &'i mut Interface,
    address: AddressType,
    register_new_with_reset: fn() -> Register,
    _phantom: PhantomData<(Register, Access)>,
}

impl<'i, Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, Register, Access>
{
    #[doc(hidden)]
    pub fn new(
        interface: &'i mut Interface,
        address: AddressType,
        register_new_with_reset: fn() -> Register,
    ) -> Self {
        Self {
            interface,
            address,
            register_new_with_reset,
            _phantom: PhantomData,
        }
    }

    /// Get the register's address.
    pub fn address(&self) -> AddressType
    where
        AddressType: Copy,
    {
        self.address
    }

    /// Get the register's reset value.
    pub fn reset_value(&self) -> Register {
        (self.register_new_with_reset)()
    }
}

impl<Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, Register, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [Self::write_with_zero].
    pub fn write<R>(&mut self, f: impl FnOnce(&mut Register) -> R) -> Result<R, Interface::Error> {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface.write_register(
            self.address,
            Register::SIZE_BITS,
            register.get_inner_buffer(),
        )?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero<R>(
        &mut self,
        f: impl FnOnce(&mut Register) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = Register::default();
        let returned = f(&mut register);
        self.interface.write_register(
            self.address,
            Register::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }
}

impl<Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, Register, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read the register from the device
    pub fn read(&mut self) -> Result<Register, Interface::Error> {
        let mut register = Register::default();

        self.interface.read_register(
            self.address,
            Register::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(register)
    }
}

impl<Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, Register, Access>
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
        self.interface.write_register(
            self.address,
            Register::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }
}

impl<Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, Register, Access>
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
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface
            .write_register(
                self.address,
                Register::SIZE_BITS,
                register.get_inner_buffer(),
            )
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
        let mut register = Register::default();
        let returned = f(&mut register);
        self.interface
            .write_register(
                self.address,
                Register::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(returned)
    }
}

impl<Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, Register, Access>
where
    Interface: AsyncRegisterInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read the register from the device
    pub async fn read_async(&mut self) -> Result<Register, Interface::Error> {
        let mut register = Register::default();

        self.interface
            .read_register(
                self.address,
                Register::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(register)
    }
}

impl<Interface, AddressType: Copy, Register: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, Register, Access>
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
            .write_register(
                self.address,
                Register::SIZE_BITS,
                register.get_inner_buffer(),
            )
            .await?;
        Ok(returned)
    }
}
