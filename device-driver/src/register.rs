use core::marker::PhantomData;

use crate::{
    Address, AddressMode, Append, Block, Fieldset, FieldsetMetadata, RO, RW, ReadCapability,
    ToTuple, WO, WriteCapability,
};

#[cfg(feature = "defmt")]
use defmt::panic;

/// Common properties shared by [`RegisterInterface`] & [`AsyncRegisterInterface`]
pub trait RegisterInterfaceBase {
    /// The error type
    type Error;
    /// The address type used by this interface
    type AddressType: Address;
}

/// A trait to represent the interface to the device.
///
/// This is called to write to and read from registers.
pub trait RegisterInterface: RegisterInterfaceBase {
    /// Write the given data to the register located at the given address
    fn write_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Read the register located at the given address to the given data slice
    fn read_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to asynchronously write to and read from registers.
pub trait AsyncRegisterInterface: RegisterInterfaceBase {
    /// Write the given data to the register located at the given address
    async fn write_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error>;

    /// Read the register located at the given address to the given data slice
    async fn read_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// Object that performs actions on the device in the context of a register
pub struct RegisterOperation<'b, B, RegisterFs, AddressType, Access, Repeat>
where
    B: Block,
    B::Interface: RegisterInterfaceBase<AddressType = AddressType>,
    RegisterFs: Fieldset,
    AddressType: Address,
{
    block: &'b mut B,
    address: AddressType,
    register_new_with_reset: fn() -> RegisterFs,
    _phantom: PhantomData<(RegisterFs, Access, Repeat)>,
}

impl<'b, B, RegisterFs, AddressType, Access, Repeat>
    RegisterOperation<'b, B, RegisterFs, AddressType, Access, Repeat>
where
    RegisterFs: Fieldset,
    B: Block,
    B::Interface: RegisterInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    #[doc(hidden)]
    pub fn new(
        interface: &'b mut B,
        address: AddressType,
        register_new_with_reset: fn() -> RegisterFs,
    ) -> Self {
        Self {
            block: interface,
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
    pub fn reset_value(&self) -> RegisterFs {
        (self.register_new_with_reset)()
    }

    /// Get a plan to read, write or modify for multi register transactions
    pub fn plan(&self) -> Plan<AddressType, RegisterFs, Access>
    where
        Repeat: NotRepeating,
    {
        Plan {
            address: self.address(),
            value: self.reset_value(),
            _phantom: PhantomData,
        }
    }

    /// Get a plan to read, write or modify for multi register transactions at a given index
    #[track_caller]
    pub fn plan_at(&self, index: Repeat::Index) -> Plan<AddressType, RegisterFs, Access>
    where
        Repeat: Repeating,
    {
        Plan {
            address: Repeat::calc_address(self.address, index),
            value: self.reset_value(),
            _phantom: PhantomData,
        }
    }

    /// Same as [`Self::plan`], but initialize the fieldset with all zeroes
    #[track_caller]
    pub fn plan_with_zero(&self) -> Plan<AddressType, RegisterFs, Access>
    where
        Repeat: NotRepeating,
        Access: WriteCapability,
    {
        Plan {
            address: self.address(),
            value: RegisterFs::ZERO,
            _phantom: PhantomData,
        }
    }

    /// Same as [`Self::plan_at`], but initialize the fieldset with all zeroes
    #[track_caller]
    pub fn plan_with_zero_at(&self, index: Repeat::Index) -> Plan<AddressType, RegisterFs, Access>
    where
        Repeat: Repeating,
        Access: WriteCapability,
    {
        Plan {
            address: Repeat::calc_address(self.address, index),
            value: RegisterFs::ZERO,
            _phantom: PhantomData,
        }
    }

    /// Get a plan to read, write or modify an array of registers for multi register transactions with a given start index and length
    #[track_caller]
    pub fn plan_array_at<const N: usize>(
        self,
        index: Repeat::Index,
    ) -> Plan<AddressType, [RegisterFs; N], Access>
    where
        Repeat: ArrayRepeating,
        B::RegisterAddressMode: AddressMode,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        Plan {
            address,
            value: core::array::from_fn(|_| self.reset_value()),
            _phantom: PhantomData,
        }
    }

    /// Same as [`Self::plan_array_at`], but initialize the fieldsets with all zeroes
    #[track_caller]
    pub fn plan_array_with_zero_at<const N: usize>(
        self,
        index: Repeat::Index,
    ) -> Plan<AddressType, [RegisterFs; N], Access>
    where
        Repeat: ArrayRepeating,
        B::RegisterAddressMode: AddressMode,
        Access: WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        Plan {
            address,
            value: Fieldset::ZERO,
            _phantom: PhantomData,
        }
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    #[track_caller]
    pub fn write(
        &mut self,
        f: impl FnOnce(&mut RegisterFs),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: NotRepeating,
        B::Interface: RegisterInterface,
        Access: WriteCapability,
    {
        let mut register = (self.register_new_with_reset)();
        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            self.address,
            register.as_slice(),
        )
    }

    /// Write to the register at a given index.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero_at`].
    #[track_caller]
    pub fn write_at(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut RegisterFs),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: Repeating,
        B::Interface: RegisterInterface,
        Access: WriteCapability,
    {
        let mut register = (self.register_new_with_reset)();
        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            Repeat::calc_address(self.address, index),
            register.as_slice(),
        )
    }

    /// Write to an array of register at the given index and N length.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_array_with_zero_at`].
    #[track_caller]
    pub fn write_array_at<const N: usize>(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut [RegisterFs; N]),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: ArrayRepeating,
        B::Interface: RegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = core::array::from_fn(|_| (self.register_new_with_reset)());
        f(&mut register);

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            address,
            Fieldset::as_slice(&register),
        )
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero`].
    #[track_caller]
    pub fn write_async(
        &mut self,
        f: impl FnOnce(&mut RegisterFs),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: NotRepeating,
        B::Interface: AsyncRegisterInterface,
        Access: WriteCapability,
    {
        let mut register = (self.register_new_with_reset)();
        f(&mut register);

        async move {
            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, self.address, register.as_slice())
                .await
        }
    }

    /// Write to the register at a given index.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_with_zero_at_async`].
    #[track_caller]
    pub fn write_at_async(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut RegisterFs),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: Repeating,
        B::Interface: AsyncRegisterInterface,
        Access: WriteCapability,
    {
        let mut register = (self.register_new_with_reset)();
        f(&mut register);

        let address = Repeat::calc_address(self.address, index);

        async move {
            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, address, register.as_slice())
                .await
        }
    }

    /// Write to an array of register at the given index and N length.
    ///
    /// The closure is given the write object initialized to the reset value of the register.
    /// If no reset value is specified for this register, this function is the same as [`Self::write_array_with_zero_at_async`].
    #[track_caller]
    pub fn write_array_at_async<const N: usize>(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut [RegisterFs; N]),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: ArrayRepeating,
        B::Interface: AsyncRegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = core::array::from_fn(|_| (self.register_new_with_reset)());
        f(&mut register);

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        async move {
            self.block
                .interface()
                .write_register(
                    &RegisterFs::METADATA,
                    address,
                    Fieldset::as_slice(&register),
                )
                .await
        }
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    #[track_caller]
    pub fn write_with_zero(
        &mut self,
        f: impl FnOnce(&mut RegisterFs),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: NotRepeating,
        B::Interface: RegisterInterface,
        Access: WriteCapability,
    {
        let mut register = RegisterFs::ZERO;
        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            self.address,
            register.as_slice_mut(),
        )
    }

    /// Write to the register at a given index.
    ///
    /// The closure is given the write object initialized to all zero.
    #[track_caller]
    pub fn write_with_zero_at(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut RegisterFs),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: Repeating,
        B::Interface: RegisterInterface,
        Access: WriteCapability,
    {
        let mut register = RegisterFs::ZERO;
        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            Repeat::calc_address(self.address, index),
            register.as_slice_mut(),
        )
    }

    /// Write to an array of registers at a given index and length.
    ///
    /// The closure is given the write object initialized to all zero.
    #[track_caller]
    pub fn write_array_with_zero_at<const N: usize>(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut [RegisterFs; N]),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: ArrayRepeating,
        B::Interface: RegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = <[RegisterFs; N] as Fieldset>::ZERO;
        f(&mut register);

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            address,
            register.as_slice_mut(),
        )
    }

    /// Write to the register.
    ///
    /// The closure is given the write object initialized to all zero.
    #[track_caller]
    pub fn write_with_zero_async(
        &mut self,
        f: impl FnOnce(&mut RegisterFs),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: NotRepeating,
        B::Interface: AsyncRegisterInterface,
        Access: WriteCapability,
    {
        let mut register = RegisterFs::ZERO;
        f(&mut register);

        async move {
            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, self.address, register.as_slice_mut())
                .await
        }
    }

    /// Write to the register at a given index.
    ///
    /// The closure is given the write object initialized to all zero.
    #[track_caller]
    pub fn write_with_zero_at_async(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut RegisterFs),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: Repeating,
        B::Interface: AsyncRegisterInterface,
        Access: WriteCapability,
    {
        let mut register = RegisterFs::ZERO;
        f(&mut register);

        let address = Repeat::calc_address(self.address, index);

        async move {
            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, address, register.as_slice_mut())
                .await
        }
    }

    /// Write to an array of registers at a given index and length.
    ///
    /// The closure is given the write object initialized to all zero.
    #[track_caller]
    pub fn write_array_with_zero_at_async<const N: usize>(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut [RegisterFs; N]),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: ArrayRepeating,
        B::Interface: AsyncRegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = <[RegisterFs; N] as Fieldset>::ZERO;
        f(&mut register);

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        async move {
            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, address, register.as_slice_mut())
                .await
        }
    }

    /// Read the register from the device
    #[track_caller]
    pub fn read(&mut self) -> Result<RegisterFs, <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: NotRepeating,
        B::Interface: RegisterInterface,
        Access: ReadCapability,
    {
        let mut register = RegisterFs::ZERO;

        self.block
            .interface()
            .read_register(&RegisterFs::METADATA, self.address, register.as_slice_mut())
            .map(|_| register)
    }

    /// Read the register from the device at a given index
    #[track_caller]
    pub fn read_at(
        &mut self,
        index: Repeat::Index,
    ) -> Result<RegisterFs, <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: Repeating,
        B::Interface: RegisterInterface,
        Access: ReadCapability,
    {
        let mut register = RegisterFs::ZERO;

        self.block
            .interface()
            .read_register(
                &RegisterFs::METADATA,
                Repeat::calc_address(self.address, index),
                register.as_slice_mut(),
            )
            .map(|_| register)
    }

    /// Read an array of registers from the device at a given index and length
    #[track_caller]
    pub fn read_array_at<const N: usize>(
        &mut self,
        index: Repeat::Index,
    ) -> Result<[RegisterFs; N], <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: ArrayRepeating,
        B::Interface: RegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: ReadCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = <[RegisterFs; N] as Fieldset>::ZERO;
        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        self.block
            .interface()
            .read_register(&RegisterFs::METADATA, address, register.as_slice_mut())
            .map(|_| register)
    }

    /// Read the register from the device
    #[track_caller]
    pub fn read_async(
        &mut self,
    ) -> impl Future<Output = Result<RegisterFs, <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: NotRepeating,
        B::Interface: AsyncRegisterInterface,
        Access: ReadCapability,
    {
        let mut register = RegisterFs::ZERO;

        async move {
            self.block
                .interface()
                .read_register(&RegisterFs::METADATA, self.address, register.as_slice_mut())
                .await
                .map(|_| register)
        }
    }

    /// Read the register from the device at a given index
    pub fn read_at_async(
        &mut self,
        index: Repeat::Index,
    ) -> impl Future<Output = Result<RegisterFs, <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: Repeating,
        B::Interface: AsyncRegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: ReadCapability,
    {
        let mut register = RegisterFs::ZERO;
        let address = Repeat::calc_address(self.address, index);

        async move {
            self.block
                .interface()
                .read_register(&RegisterFs::METADATA, address, register.as_slice_mut())
                .await
                .map(|_| register)
        }
    }

    /// Read an array of registers from the device at a given index and length
    pub fn read_array_at_async<const N: usize>(
        &mut self,
        index: Repeat::Index,
    ) -> impl Future<Output = Result<[RegisterFs; N], <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: ArrayRepeating,
        B::Interface: AsyncRegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: ReadCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = <[RegisterFs; N] as Fieldset>::ZERO;
        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        async move {
            self.block
                .interface()
                .read_register(&RegisterFs::METADATA, address, register.as_slice_mut())
                .await
                .map(|_| register)
        }
    }

    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    #[track_caller]
    pub fn modify(
        &mut self,
        f: impl FnOnce(&mut RegisterFs),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: NotRepeating,
        B::Interface: RegisterInterface,
        Access: ReadCapability + WriteCapability,
    {
        let mut register = RegisterFs::ZERO;

        self.block.interface().read_register(
            &RegisterFs::METADATA,
            self.address,
            register.as_slice_mut(),
        )?;

        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            self.address,
            register.as_slice_mut(),
        )
    }

    /// Modify the existing register value at a given index.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    #[track_caller]
    pub fn modify_at(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut RegisterFs),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: Repeating,
        B::Interface: RegisterInterface,
        Access: ReadCapability + WriteCapability,
    {
        let mut register = RegisterFs::ZERO;
        let address = Repeat::calc_address(self.address, index);

        self.block.interface().read_register(
            &RegisterFs::METADATA,
            address,
            register.as_slice_mut(),
        )?;

        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            address,
            register.as_slice_mut(),
        )
    }

    /// Modify an array of existing register values at a given start index and length.
    ///
    /// The registers are read, the values are then passed to the closure for making changes.
    /// The result is then written back to the device.
    #[track_caller]
    pub fn modify_array_at<const N: usize>(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut [RegisterFs; N]),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        Repeat: ArrayRepeating,
        B::Interface: RegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: ReadCapability + WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = <[RegisterFs; N] as Fieldset>::ZERO;

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        self.block.interface().read_register(
            &RegisterFs::METADATA,
            address,
            register.as_slice_mut(),
        )?;

        f(&mut register);

        self.block.interface().write_register(
            &RegisterFs::METADATA,
            address,
            Fieldset::as_slice(&register),
        )
    }

    /// Modify the existing register value.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    #[track_caller]
    pub fn modify_async(
        &mut self,
        f: impl FnOnce(&mut RegisterFs),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: NotRepeating,
        B::Interface: AsyncRegisterInterface,
        Access: ReadCapability + WriteCapability,
    {
        let mut register = RegisterFs::ZERO;

        async move {
            self.block
                .interface()
                .read_register(&RegisterFs::METADATA, self.address, register.as_slice_mut())
                .await?;

            f(&mut register);

            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, self.address, register.as_slice())
                .await
        }
    }

    /// Modify the existing register value at a given index.
    ///
    /// The register is read, the value is then passed to the closure for making changes.
    /// The result is then written back to the device.
    #[track_caller]
    pub fn modify_at_async(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut RegisterFs),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: Repeating,
        B::Interface: AsyncRegisterInterface,
        Access: ReadCapability + WriteCapability,
    {
        let mut register = RegisterFs::ZERO;
        let address = Repeat::calc_address(self.address, index);

        async move {
            self.block
                .interface()
                .read_register(&RegisterFs::METADATA, address, register.as_slice_mut())
                .await?;

            f(&mut register);

            self.block
                .interface()
                .write_register(&RegisterFs::METADATA, address, register.as_slice())
                .await
        }
    }

    /// Modify an array of existing register values at a given starting index and length.
    ///
    /// The registers are read, the values are then passed to the closure for making changes.
    /// The result is then written back to the device.
    #[track_caller]
    pub fn modify_array_at_async<const N: usize>(
        &mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut [RegisterFs; N]),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        Repeat: ArrayRepeating,
        B::Interface: AsyncRegisterInterface,
        B::RegisterAddressMode: AddressMode,
        Access: ReadCapability + WriteCapability,
    {
        Repeat::assert_len_and_index(N, index.clone());

        let mut register = <[RegisterFs; N] as Fieldset>::ZERO;

        let address = Repeat::calc_address(self.address, index);
        Self::assert_array_op_legal(address);

        async move {
            self.block
                .interface()
                .read_register(&RegisterFs::METADATA, address, register.as_slice_mut())
                .await?;

            f(&mut register);

            self.block
                .interface()
                .write_register(
                    &RegisterFs::METADATA,
                    address,
                    Fieldset::as_slice(&register),
                )
                .await
        }
    }

    #[track_caller]
    fn assert_array_op_legal(address: AddressType)
    where
        B::RegisterAddressMode: AddressMode,
        Repeat: ArrayRepeating,
    {
        if address.add(Repeat::STRIDE)
            != B::RegisterAddressMode::next_address(address, core::mem::size_of::<RegisterFs>())
        {
            panic!(
                "array operations can't be used with this register due to the `register-address-map` rule. Used stride: {}, accepted stride: {}",
                Repeat::STRIDE,
                B::RegisterAddressMode::next_address(
                    AddressType::ZERO,
                    core::mem::size_of::<RegisterFs>()
                )
            );
        }
    }
}

/// A plan that is used for multi-reads and writes.
pub struct Plan<AddressType: Copy, FS, Access> {
    /// The address of the register
    pub address: AddressType,
    /// The starting value of the register. This is either the reset value or all-0's
    pub value: FS,
    _phantom: PhantomData<Access>,
}

/// A register operation for reading or writing multiple registers in one transaction
pub struct MultiRegisterOperation<'b, B, AddressType: Address, Fieldsets, Access> {
    pub(crate) block: &'b mut B,
    pub(crate) start_address: Option<AddressType>,
    pub(crate) next_address: Option<AddressType>,
    pub(crate) field_sets: Fieldsets,
    pub(crate) _phantom: PhantomData<Access>,
}

impl<B, AddressType, FieldSets, Access>
    MultiRegisterOperation<'_, B, AddressType, FieldSets, Access>
where
    B: Block,
    B::RegisterAddressMode: AddressMode,
    AddressType: Address,
{
    #[track_caller]
    fn assert_legal(&self, address: AddressType) {
        if let Some(next_address) = self.next_address
            && address != next_address
        {
            panic!(
                "order of registers not valid according to the address mode rules. Expected address: {}, got: {}",
                next_address, address
            );
        }
    }
}

impl<'b, B, AddressType, FieldSets> MultiRegisterOperation<'b, B, AddressType, FieldSets, WO>
where
    B: Block,
    B::RegisterAddressMode: AddressMode,
    AddressType: Address,
{
    /// Chain an extra write onto the multi-write.
    ///
    /// The closure must return a plan for the register you want to write.
    /// The plan is created by calling [`RegisterOperation::plan`] or [`RegisterOperation::plan_with_zero`].
    ///
    /// After chaining, call [`Self::execute`].
    #[track_caller]
    #[inline]
    pub fn with<FS: Fieldset, LocalAccess: WriteCapability>(
        self,
        f: impl FnOnce(&mut B) -> Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'b, B, AddressType, FieldSets::Appended, WO>
    where
        FieldSets: Append<FS>,
    {
        let Plan { address, value, .. } = f(self.block);
        self.assert_legal(address);

        MultiRegisterOperation {
            block: self.block,
            start_address: self.start_address.or(Some(address)),
            next_address: Some(B::RegisterAddressMode::next_address(
                address,
                core::mem::size_of::<FS>(),
            )),
            field_sets: self.field_sets.append(value),
            _phantom: PhantomData,
        }
    }
}

impl<'b, B, AddressType, FieldSets> MultiRegisterOperation<'b, B, AddressType, FieldSets, RO>
where
    B: Block,
    B::RegisterAddressMode: AddressMode,
    AddressType: Address,
{
    /// Chain an extra read onto the multi-read.
    ///
    /// The closure must return a plan for the register you want to read.
    /// The plan is created by calling [`RegisterOperation::plan`].
    ///
    /// After chaining, call [`Self::execute`].
    #[track_caller]
    #[inline]
    pub fn with<FS: Fieldset, LocalAccess: ReadCapability>(
        self,
        f: impl FnOnce(&mut B) -> Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'b, B, AddressType, FieldSets::Appended, RO>
    where
        FieldSets: Append<FS>,
    {
        let Plan { address, value, .. } = f(self.block);
        self.assert_legal(address);

        MultiRegisterOperation {
            block: self.block,
            start_address: self.start_address.or(Some(address)),
            next_address: Some(B::RegisterAddressMode::next_address(
                address,
                core::mem::size_of::<FS>(),
            )),
            field_sets: self.field_sets.append(value),
            _phantom: PhantomData,
        }
    }
}

impl<'b, B, AddressType, FieldSets> MultiRegisterOperation<'b, B, AddressType, FieldSets, RW>
where
    B: Block,
    B::RegisterAddressMode: AddressMode,
    AddressType: Address,
{
    /// Chain an extra modify onto the multi-modify.
    ///
    /// The closure must return a plan for the register you want to modify.
    /// The plan is created by calling [`RegisterOperation::plan`].
    ///
    /// After chaining, call [`Self::execute`].
    #[track_caller]
    #[inline]
    pub fn with<FS: Fieldset, LocalAccess: ReadCapability + WriteCapability>(
        self,
        f: impl FnOnce(&mut B) -> Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'b, B, AddressType, FieldSets::Appended, RW>
    where
        FieldSets: Append<FS>,
    {
        let Plan { address, value, .. } = f(self.block);
        self.assert_legal(address);

        MultiRegisterOperation {
            block: self.block,
            start_address: self.start_address.or(Some(address)),
            next_address: Some(B::RegisterAddressMode::next_address(
                address,
                core::mem::size_of::<FS>(),
            )),
            field_sets: self.field_sets.append(value),
            _phantom: PhantomData,
        }
    }
}

impl<B, Fieldsets>
    MultiRegisterOperation<
        '_,
        B,
        <B::Interface as RegisterInterfaceBase>::AddressType,
        Fieldsets,
        RO,
    >
where
    B: Block,
    B::Interface: RegisterInterfaceBase,
    Fieldsets: Fieldset + ToTuple,
{
    /// Execute the read.
    ///
    /// If ok, the fieldset values are returned as a tuple.
    /// If the multi-read was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute(
        mut self,
    ) -> Result<Fieldsets::Tuple, <B::Interface as RegisterInterfaceBase>::Error>
    where
        B::Interface: RegisterInterface,
    {
        self.block
            .interface()
            .read_register(
                &Fieldsets::METADATA,
                self.start_address.unwrap(),
                self.field_sets.as_slice_mut(),
            )
            .map(|_| self.field_sets.to_tuple())
    }

    /// Execute the read.
    ///
    /// If ok, the fieldset values are returned as a tuple.
    /// If the multi-read was illegal or the read failed, an error is returned.
    #[inline]
    pub async fn execute_async(
        mut self,
    ) -> Result<Fieldsets::Tuple, <B::Interface as RegisterInterfaceBase>::Error>
    where
        B::Interface: AsyncRegisterInterface,
    {
        self.block
            .interface()
            .read_register(
                &Fieldsets::METADATA,
                self.start_address.unwrap(),
                self.field_sets.as_slice_mut(),
            )
            .await
            .map(|_| self.field_sets.to_tuple())
    }
}

impl<B, Fieldsets>
    MultiRegisterOperation<
        '_,
        B,
        <B::Interface as RegisterInterfaceBase>::AddressType,
        Fieldsets,
        WO,
    >
where
    B: Block,
    B::Interface: RegisterInterfaceBase,
    Fieldsets: Fieldset,
    for<'a> &'a mut Fieldsets: ToTuple,
{
    /// Execute the write.
    ///
    /// Use the closure to change contents of the fieldset values that will be written.
    /// The fieldset values are either the reset value or all-0's based on which plan was used in the chaining phase.
    ///
    /// If ok, the return value of the closure is returned.
    /// If the multi-write was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute(
        mut self,
        f: impl FnOnce(<&mut Fieldsets as ToTuple>::Tuple),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        B::Interface: RegisterInterface,
    {
        f(self.field_sets.to_tuple());

        self.block.interface().write_register(
            &Fieldsets::METADATA,
            self.start_address.unwrap(),
            self.field_sets.as_slice(),
        )
    }

    /// Execute the write.
    ///
    /// Use the closure to change contents of the fieldset values that will be written.
    /// The fieldset values are either the reset value or all-0's based on which plan was used in the chaining phase.
    ///
    /// If ok, the return value of the closure is returned.
    /// If the multi-write was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute_async(
        mut self,
        f: impl FnOnce(<&mut Fieldsets as ToTuple>::Tuple),
    ) -> impl Future<Output = Result<(), <B::Interface as RegisterInterfaceBase>::Error>>
    where
        B::Interface: AsyncRegisterInterface,
    {
        f(self.field_sets.to_tuple());

        async move {
            self.block
                .interface()
                .write_register(
                    &Fieldsets::METADATA,
                    self.start_address.unwrap(),
                    self.field_sets.as_slice(),
                )
                .await
        }
    }
}

impl<B, Fieldsets>
    MultiRegisterOperation<
        '_,
        B,
        <B::Interface as RegisterInterfaceBase>::AddressType,
        Fieldsets,
        RW,
    >
where
    B: Block,
    B::Interface: RegisterInterfaceBase,
    Fieldsets: Fieldset,
    for<'a> &'a mut Fieldsets: ToTuple,
{
    /// Execute the modify.
    ///
    /// Use the closure to change contents of the fieldset values that have been read.
    /// The modified values will be written back to the device.
    ///
    /// If ok, the return value of the closure is returned.
    /// If the multi-modify was illegal or the read failed, an error is returned.
    #[inline]
    pub fn execute(
        mut self,
        f: impl FnOnce(<&mut Fieldsets as ToTuple>::Tuple),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        B::Interface: RegisterInterface,
    {
        self.block.interface().read_register(
            &Fieldsets::METADATA,
            self.start_address.unwrap(),
            self.field_sets.as_slice_mut(),
        )?;

        f(self.field_sets.to_tuple());

        self.block.interface().write_register(
            &Fieldsets::METADATA,
            self.start_address.unwrap(),
            self.field_sets.as_slice(),
        )
    }

    /// Execute the modify.
    ///
    /// Use the closure to change contents of the fieldset values that have been read.
    /// The modified values will be written back to the device.
    ///
    /// If ok, the return value of the closure is returned.
    /// If the multi-modify was illegal or the read failed, an error is returned.
    #[inline]
    pub async fn execute_async(
        mut self,
        f: impl FnOnce(<&mut Fieldsets as ToTuple>::Tuple),
    ) -> Result<(), <B::Interface as RegisterInterfaceBase>::Error>
    where
        B::Interface: AsyncRegisterInterface,
    {
        self.block
            .interface()
            .read_register(
                &Fieldsets::METADATA,
                self.start_address.unwrap(),
                self.field_sets.as_slice_mut(),
            )
            .await?;

        f(self.field_sets.to_tuple());

        self.block
            .interface()
            .write_register(
                &Fieldsets::METADATA,
                self.start_address.unwrap(),
                self.field_sets.as_slice(),
            )
            .await
    }
}

#[diagnostic::on_unimplemented(
    label = "this register does not repeat. Use the function variant without `_at` in the name to interact with the register"
)]
#[doc(hidden)]
pub trait Repeating {
    type Index: Clone;

    /// Calculate an address with the index
    #[allow(private_bounds)]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType;
}

#[diagnostic::on_unimplemented(
    label = "this register has a repeat and you must specify an index. Use the function variant with `_at` in the name to interact with the register"
)]
#[doc(hidden)]
pub trait NotRepeating {}
impl NotRepeating for () {}

#[diagnostic::on_unimplemented(
    label = "this register has a repeat, but can't be used with array operations. Avoid using functions with `_array` in the name to interact with the register",
    note = "repeats that use an enum cannot be used as an array"
)]
#[doc(hidden)]
pub trait ArrayRepeating: Repeating {
    const COUNT: u16;
    const STRIDE: i32;

    fn assert_len_and_index(len: usize, index: Self::Index);
}

#[doc(hidden)]
pub struct ArrayRepeat<const COUNT: u16, const STRIDE: i32>;
impl<const COUNT: u16, const STRIDE: i32> Repeating for ArrayRepeat<COUNT, STRIDE> {
    type Index = usize;

    #[track_caller]
    #[inline]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        assert!(
            index < COUNT as usize,
            "Index out of range: {index} (array len: {COUNT})"
        );
        let offset = index as i32 * STRIDE;
        start.add(offset)
    }
}
impl<const COUNT: u16, const STRIDE: i32> ArrayRepeating for ArrayRepeat<COUNT, STRIDE> {
    const COUNT: u16 = COUNT;
    const STRIDE: i32 = STRIDE;

    #[track_caller]
    #[inline]
    fn assert_len_and_index(len: usize, index: Self::Index) {
        assert!(
            index < COUNT as usize,
            "index out of range: {index} (array len: {COUNT})"
        );
        assert!(
            len + index <= COUNT as usize,
            "array too long. Requested {len}, max len remaining at requested index is {}",
            COUNT as usize - index,
        );
    }
}

#[doc(hidden)]
pub struct EnumRepeat<T, const STRIDE: i32>(PhantomData<T>);
impl<T: Clone + Into<i32>, const STRIDE: i32> Repeating for EnumRepeat<T, STRIDE> {
    type Index = T;

    #[inline]
    fn calc_address<AddressType: Address>(start: AddressType, index: Self::Index) -> AddressType {
        let offset = index.into() * STRIDE;
        start.add(offset)
    }
}
