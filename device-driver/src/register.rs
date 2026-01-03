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

    /// Read the register located at the given address to the given data slice
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

    /// Read the register located at the given address to the given data slice
    async fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// Object that performs actions on the device in the context of a register
pub struct RegisterOperation<'i, Interface, AddressType: Copy, FS: FieldSet, Access> {
    interface: &'i mut Interface,
    address: AddressType,
    register_new_with_reset: fn() -> FS,
    _phantom: PhantomData<(FS, Access)>,
}

impl<'i, Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'i, Interface, AddressType, FS, Access>
{
    #[doc(hidden)]
    pub fn new(
        interface: &'i mut Interface,
        address: AddressType,
        register_new_with_reset: fn() -> FS,
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
    pub fn reset_value(&self) -> FS {
        (self.register_new_with_reset)()
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Access: WriteCapability,
{
    pub fn plan_write(
        &mut self,
        f: impl FnOnce(&mut FS),
    ) -> Plan<AddressType, FS, crate::WO> {
        let mut register = (self.register_new_with_reset)();
        f(&mut register);

        Plan {
            address: self.address(),
            value: register,
            _phantom: PhantomData,
        }
    }

    pub fn plan_write_with_zero(
        &mut self,
        f: impl FnOnce(&mut FS),
    ) -> Plan<AddressType, FS, crate::WO> {
        let mut register = FS::default();
        f(&mut register);

        Plan {
            address: self.address(),
            value: register,
            _phantom: PhantomData,
        }
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Access: ReadCapability,
{
    pub fn plan_read(&mut self) -> Plan<AddressType, FS, crate::RO> {
        Plan {
            address: self.address(),
            value: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Access: ReadCapability + WriteCapability,
{
    pub fn plan_modify(&mut self) -> Plan<AddressType, FS, crate::RW> {
        Plan {
            address: self.address(),
            value: Default::default(),
            _phantom: PhantomData,
        }
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    pub fn write<R>(&mut self, f: impl FnOnce(&mut FS) -> R) -> Result<R, Interface::Error> {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface.write_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer(),
        )?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero<R>(
        &mut self,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = FS::default();
        let returned = f(&mut register);
        self.interface.write_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read the register from the device
    pub fn read(&mut self) -> Result<FS, Interface::Error> {
        let mut register = FS::default();

        self.interface.read_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(register)
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Interface: RegisterInterface<AddressType = AddressType>,
    Access: ReadCapability + WriteCapability,
{
    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub fn modify<R>(&mut self, f: impl FnOnce(&mut FS) -> R) -> Result<R, Interface::Error> {
        let mut register = self.read()?;
        let returned = f(&mut register);
        self.interface.write_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Interface: AsyncRegisterInterface<AddressType = AddressType>,
    Access: WriteCapability,
{
    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    pub async fn write_async<R>(
        &mut self,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface
            .write_register(
                self.address,
                FS::SIZE_BITS,
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
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = FS::default();
        let returned = f(&mut register);
        self.interface
            .write_register(
                self.address,
                FS::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(returned)
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
where
    Interface: AsyncRegisterInterface<AddressType = AddressType>,
    Access: ReadCapability,
{
    /// Read the register from the device
    pub async fn read_async(&mut self) -> Result<FS, Interface::Error> {
        let mut register = FS::default();

        self.interface
            .read_register(
                self.address,
                FS::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(register)
    }
}

impl<Interface, AddressType: Copy, FS: FieldSet, Access>
    RegisterOperation<'_, Interface, AddressType, FS, Access>
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
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error> {
        let mut register = self.read_async().await?;
        let returned = f(&mut register);
        self.interface
            .write_register(
                self.address,
                FS::SIZE_BITS,
                register.get_inner_buffer(),
            )
            .await?;
        Ok(returned)
    }
}

pub struct Plan<AddressType: Copy, FS: FieldSet, Access> {
    pub address: AddressType,
    pub value: FS,
    _phantom: PhantomData<Access>,
}
