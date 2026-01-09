use core::marker::PhantomData;

use crate::{
    Address, Block, FieldSet, FsSet, NotRepeating, RO, RW, ReadCapability, Repeating, WO,
    WriteCapability,
};

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
pub struct RegisterOperation<'i, Interface, AddressType: Address, FS, Access, Form> {
    interface: &'i mut Interface,
    address: AddressType,
    register_new_with_reset: fn() -> FS,
    _phantom: PhantomData<(FS, Access, Form)>,
}

impl<'i, Interface, AddressType, FS, Access, Form>
    RegisterOperation<'i, Interface, AddressType, FS, Access, Form>
where
    AddressType: Address,
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
    pub fn address(&self) -> AddressType {
        self.address
    }

    /// Get the register's reset value.
    pub fn reset_value(&self) -> FS {
        (self.register_new_with_reset)()
    }
}

impl<Interface, AddressType, FS, Access, Form>
    RegisterOperation<'_, Interface, AddressType, FS, Access, Form>
where
    AddressType: Address,
    FS: FieldSet,
{
    /// Get a plan with the register value set to all 0's.
    /// It's the plan equivalent of [Self::write_with_zero].
    pub fn plan_with_zero(self) -> Plan<AddressType, FS, WO>
    where
        Access: WriteCapability,
        Form: NotRepeating,
    {
        Plan {
            address: self.address(),
            value: FS::default(),
            _phantom: PhantomData,
        }
    }

    /// Get a plan with the register value set to all 0's.
    /// It's the plan equivalent of [Self::write_with_zero].
    pub fn plan_with_zero_at(self, index: Form::Index) -> Plan<AddressType, FS, WO>
    where
        Access: WriteCapability,
        Form: Repeating,
    {
        Plan {
            address: Form::calc_address(self.address(), index),
            value: FS::default(),
            _phantom: PhantomData,
        }
    }

    /// Get a plan with the reset value of the register
    pub fn plan(self) -> Plan<AddressType, FS, Access>
    where
        Form: NotRepeating,
    {
        Plan {
            address: self.address(),
            value: self.reset_value(),
            _phantom: PhantomData,
        }
    }

    /// Get a plan with the reset value of the register
    pub fn plan_at(self, index: Form::Index) -> Plan<AddressType, FS, Access>
    where
        Form: Repeating,
    {
        Plan {
            address: Form::calc_address(self.address(), index),
            value: self.reset_value(),
            _phantom: PhantomData,
        }
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    pub fn write<R>(&mut self, f: impl FnOnce(&mut FS) -> R) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface
            .write_register(self.address, FS::SIZE_BITS, register.get_inner_buffer())?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    pub fn write_at<R>(
        &mut self,
        index: Form::Index,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface.write_register(
            Form::calc_address(self.address, index),
            FS::SIZE_BITS,
            register.get_inner_buffer(),
        )?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    pub async fn write_async<R>(
        &mut self,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface
            .write_register(self.address, FS::SIZE_BITS, register.get_inner_buffer())
            .await?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    pub async fn write_at_async<R>(
        &mut self,
        index: Form::Index,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = (self.register_new_with_reset)();
        let returned = f(&mut register);

        self.interface
            .write_register(
                Form::calc_address(self.address, index),
                FS::SIZE_BITS,
                register.get_inner_buffer(),
            )
            .await?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero<R>(
        &mut self,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = FS::default();
        let returned = f(&mut register);
        self.interface.write_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub fn write_with_zero_at<R>(
        &mut self,
        index: Form::Index,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = FS::default();
        let returned = f(&mut register);
        self.interface.write_register(
            Form::calc_address(self.address, index),
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub async fn write_with_zero_async<R>(
        &mut self,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = FS::default();
        let returned = f(&mut register);
        self.interface
            .write_register(self.address, FS::SIZE_BITS, register.get_inner_buffer_mut())
            .await?;
        Ok(returned)
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    pub async fn write_with_zero_at_async<R>(
        &mut self,
        index: Form::Index,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: WriteCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = FS::default();
        let returned = f(&mut register);
        self.interface
            .write_register(
                Form::calc_address(self.address, index),
                FS::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(returned)
    }

    /// Read the register from the device
    pub fn read(&mut self) -> Result<FS, Interface::Error>
    where
        Access: ReadCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = FS::default();

        self.interface.read_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(register)
    }

    /// Read the register from the device
    pub fn read_at(&mut self, index: Form::Index) -> Result<FS, Interface::Error>
    where
        Access: ReadCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = FS::default();

        self.interface.read_register(
            Form::calc_address(self.address, index),
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(register)
    }

    /// Read the register from the device
    pub async fn read_async(&mut self) -> Result<FS, Interface::Error>
    where
        Access: ReadCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = FS::default();

        self.interface
            .read_register(self.address, FS::SIZE_BITS, register.get_inner_buffer_mut())
            .await?;
        Ok(register)
    }

    /// Read the register from the device
    pub async fn read_at_async(&mut self, index: Form::Index) -> Result<FS, Interface::Error>
    where
        Access: ReadCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = FS::default();

        self.interface
            .read_register(
                Form::calc_address(self.address, index),
                FS::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(register)
    }

    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub fn modify<R>(&mut self, f: impl FnOnce(&mut FS) -> R) -> Result<R, Interface::Error>
    where
        Access: ReadCapability + WriteCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = self.read()?;
        let returned = f(&mut register);
        self.interface.write_register(
            self.address,
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }

    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub fn modify_at<R>(
        &mut self,
        index: Form::Index,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: ReadCapability + WriteCapability,
        Interface: RegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = self.read_at(index.clone())?;
        let returned = f(&mut register);
        self.interface.write_register(
            Form::calc_address(self.address, index),
            FS::SIZE_BITS,
            register.get_inner_buffer_mut(),
        )?;
        Ok(returned)
    }

    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub async fn modify_async<R>(
        &mut self,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: ReadCapability + WriteCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut register = self.read_async().await?;
        let returned = f(&mut register);
        self.interface
            .write_register(self.address, FS::SIZE_BITS, register.get_inner_buffer_mut())
            .await?;
        Ok(returned)
    }

    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    pub async fn modify_at_async<R>(
        &mut self,
        index: Form::Index,
        f: impl FnOnce(&mut FS) -> R,
    ) -> Result<R, Interface::Error>
    where
        Access: ReadCapability + WriteCapability,
        Interface: AsyncRegisterInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut register = self.read_at_async(index.clone()).await?;
        let returned = f(&mut register);
        self.interface
            .write_register(
                Form::calc_address(self.address, index),
                FS::SIZE_BITS,
                register.get_inner_buffer_mut(),
            )
            .await?;
        Ok(returned)
    }
}

/// A plan that is used for multi-reads and writes.
pub struct Plan<AddressType: Copy, FS: FieldSet, Access> {
    /// The address of the register
    pub address: AddressType,
    /// The starting value of the register. This is either the reset value or all-0's
    pub value: FS,
    _phantom: PhantomData<Access>,
}

/// A register operation for reading or writing multiple registers in one transaction
pub struct MultiRegisterOperation<'d, D, AddressType, FieldSets: FsSet, Access> {
    pub(crate) device: &'d mut D,
    pub(crate) start_address: Option<AddressType>,
    pub(crate) field_sets: FieldSets,
    pub(crate) bit_sum: u32,
    pub(crate) _phantom: PhantomData<Access>,
}

impl<'d, D, AddressType, FieldSets: FsSet> MultiRegisterOperation<'d, D, AddressType, FieldSets, RO>
where
    D: Block,
    AddressType: Copy,
{
    /// Chain an extra read onto the multi-read.
    ///
    /// The closure must return a plan for the register you want to read.
    /// The plan is created by calling [RegisterOperation::plan].
    ///
    /// After chaining, call [Self::execute].
    #[inline]
    pub fn with<FS: crate::FieldSet, LocalAccess: ReadCapability>(
        mut self,
        f: impl FnOnce(&mut D) -> crate::Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'d, D, AddressType, FieldSets::Next<FS>, RO>
    where
        FieldSets::Next<FS>: FsSet,
    {
        let Plan { address, value, .. } = f(self.device);

        if self.start_address.is_none() {
            self.start_address = Some(address)
        }
        assert!(FS::SIZE_BITS.is_multiple_of(8));

        // TODO: Check if legal

        MultiRegisterOperation {
            device: self.device,
            start_address: self.start_address,
            field_sets: self.field_sets.push(value),
            bit_sum: self.bit_sum + FS::SIZE_BITS,
            _phantom: PhantomData,
        }
    }
}

impl<'d, D, AddressType, FieldSets: FsSet> MultiRegisterOperation<'d, D, AddressType, FieldSets, WO>
where
    D: Block,
    AddressType: Copy,
{
    /// Chain an extra write onto the multi-write.
    ///
    /// The closure must return a plan for the register you want to write.
    /// The plan is created by calling [RegisterOperation::plan] or [RegisterOperation::plan_with_zero].
    ///
    /// After chaining, call [Self::execute].
    #[inline]
    pub fn with<FS: crate::FieldSet, LocalAccess: WriteCapability>(
        mut self,
        f: impl FnOnce(&mut D) -> crate::Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'d, D, AddressType, FieldSets::Next<FS>, WO>
    where
        FieldSets::Next<FS>: FsSet,
    {
        let Plan { address, value, .. } = f(self.device);

        if self.start_address.is_none() {
            self.start_address = Some(address)
        }
        assert!(FS::SIZE_BITS.is_multiple_of(8));

        // TODO: Check if legal

        MultiRegisterOperation {
            device: self.device,
            start_address: self.start_address,
            field_sets: self.field_sets.push(value),
            bit_sum: self.bit_sum + FS::SIZE_BITS,
            _phantom: PhantomData,
        }
    }
}

impl<'d, D, AddressType, FieldSets: FsSet> MultiRegisterOperation<'d, D, AddressType, FieldSets, RW>
where
    D: Block,
    AddressType: Copy,
{
    /// Chain an extra modify onto the multi-modify.
    ///
    /// The closure must return a plan for the register you want to modify.
    /// The plan is created by calling [RegisterOperation::plan].
    ///
    /// After chaining, call [Self::execute].
    #[inline]
    pub fn with<FS: crate::FieldSet, LocalAccess: WriteCapability + ReadCapability>(
        mut self,
        f: impl FnOnce(&mut D) -> crate::Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'d, D, AddressType, FieldSets::Next<FS>, RW>
    where
        FieldSets::Next<FS>: FsSet,
    {
        let Plan { address, value, .. } = f(self.device);

        if self.start_address.is_none() {
            self.start_address = Some(address)
        }
        assert!(FS::SIZE_BITS.is_multiple_of(8));

        // TODO: Check if legal

        MultiRegisterOperation {
            device: self.device,
            start_address: self.start_address,
            field_sets: self.field_sets.push(value),
            bit_sum: self.bit_sum + FS::SIZE_BITS,
            _phantom: PhantomData,
        }
    }
}

impl<'d, D, FieldSets: FsSet>
    MultiRegisterOperation<
        'd,
        D,
        <D::Interface as crate::RegisterInterface>::AddressType,
        FieldSets,
        RO,
    >
where
    D: Block,
    D::Interface: crate::RegisterInterface,
{
    /// Execute the read.
    ///
    /// If ok, the fieldset values are returned as a tuple.
    /// If the multi-read was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute(
        mut self,
    ) -> Result<FieldSets::Value, <D::Interface as crate::RegisterInterface>::Error> {
        let data = self.field_sets.as_slice_mut();
        self.device
            .interface()
            .read_register(self.start_address.unwrap(), self.bit_sum, data)?;
        Ok(self.field_sets.to_value())
    }
}

impl<'d, D, FieldSets: FsSet>
    MultiRegisterOperation<
        'd,
        D,
        <D::Interface as crate::RegisterInterface>::AddressType,
        FieldSets,
        WO,
    >
where
    D: Block,
    D::Interface: crate::RegisterInterface,
{
    /// Execute the write.
    ///
    /// Use the closure to change contents of the fieldset values that will be written.
    /// The fieldset values are either the reset value or all-0's based on which plan was used in the chaining phase.
    ///
    /// If ok, the return value of the closure is returned.
    /// If the multi-write was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute<R>(
        mut self,
        f: impl FnOnce(FieldSets::ValueMut<'_>) -> R,
    ) -> Result<R, <D::Interface as crate::RegisterInterface>::Error> {
        let returned = f(self.field_sets.as_value_mut());
        self.device.interface().write_register(
            self.start_address.unwrap(),
            self.bit_sum,
            self.field_sets.as_slice_mut(),
        )?;
        Ok(returned)
    }
}

impl<'d, D, FieldSets: FsSet>
    MultiRegisterOperation<
        'd,
        D,
        <D::Interface as crate::RegisterInterface>::AddressType,
        FieldSets,
        RW,
    >
where
    D: Block,
    D::Interface: crate::RegisterInterface,
{
    /// Execute the modify.
    ///
    /// Use the closure to change contents of the fieldset values that have been read.
    /// The modified values will be written back to the device.
    ///
    /// If ok, the return value of the closure is returned.
    /// If the multi-modify was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute<R>(
        mut self,
        f: impl FnOnce(FieldSets::ValueMut<'_>) -> R,
    ) -> Result<R, <D::Interface as crate::RegisterInterface>::Error> {
        self.device.interface().read_register(
            self.start_address.unwrap(),
            self.bit_sum,
            self.field_sets.as_slice_mut(),
        )?;

        let returned = f(self.field_sets.as_value_mut());

        self.device.interface().write_register(
            self.start_address.unwrap(),
            self.bit_sum,
            self.field_sets.as_slice_mut(),
        )?;

        Ok(returned)
    }
}
