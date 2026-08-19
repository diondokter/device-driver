use core::marker::PhantomData;

use crate::{Address, Block, Fieldset, FieldsetMetadata};

/// Common properties shared by [`CommandInterface`] & [`AsyncCommandInterface`]
pub trait CommandInterfaceBase {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Address;
}

impl<T: CommandInterfaceBase> CommandInterfaceBase for &mut T {
    type Error = T::Error;
    type AddressType = T::AddressType;
}

#[diagnostic::on_unimplemented(
    label = "cannot use blocking command operations when the device interface doesn't know how to dispatch commands",
    note = "to enable command operations, implement the trait on this type"
)]
/// A trait to represent the interface to the device.
///
/// This is called to dispatch commands.
pub trait CommandInterface: CommandInterfaceBase {
    /// Dispatch a command on the device by sending the command.
    ///
    /// The input is the content that needs to be sent to the device.
    /// The output is the buffer where the response needs to be written to.
    ///
    /// The slices are empty if the respective in or out fields are not specified.
    fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        input: &mut [u8],
        _input_metadata: &FieldsetMetadata,
        output: &mut [u8],
        _output_metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error>;
}

#[diagnostic::do_not_recommend]
impl<T: CommandInterface> CommandInterface for &mut T {
    fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        input: &mut [u8],
        input_metadata: &FieldsetMetadata,
        output: &mut [u8],
        output_metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        (*self).dispatch_command(address, input, input_metadata, output, output_metadata)
    }
}

#[diagnostic::on_unimplemented(
    label = "cannot use async command operations when the device interface doesn't know how to dispatch commands",
    note = "to enable command operations, implement the trait on this type"
)]
/// A trait to represent the interface to the device.
///
/// This is called to asynchronously dispatch commands.
pub trait AsyncCommandInterface: CommandInterfaceBase {
    /// Dispatch a command on the device by sending the command.
    ///
    /// The input is the content that needs to be sent to the device.
    /// The output is the buffer where the response needs to be written to.
    ///
    /// The slices are empty if the respective in or out fields are not specified.
    async fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        input: &mut [u8],
        _input_metadata: &FieldsetMetadata,
        output: &mut [u8],
        _output_metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error>;
}

#[diagnostic::do_not_recommend]
impl<T: AsyncCommandInterface> AsyncCommandInterface for &mut T {
    fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        input: &mut [u8],
        input_metadata: &FieldsetMetadata,
        output: &mut [u8],
        output_metadata: &FieldsetMetadata,
    ) -> impl Future<Output = Result<(), Self::Error>> {
        (*self).dispatch_command(address, input, input_metadata, output, output_metadata)
    }
}

/// Intermediate type for doing command operations
pub struct CommandOperation<'b, B, AddressType, InFieldset, OutFieldset>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    block: &'b mut B,
    address: AddressType,
    _phantom: PhantomData<(InFieldset, OutFieldset)>,
}

impl<'d, B, AddressType, InFieldset, OutFieldset>
    CommandOperation<'d, B, AddressType, InFieldset, OutFieldset>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    #[doc(hidden)]
    pub fn new(block: &'d mut B, address: AddressType) -> Self {
        Self {
            block,
            address,
            _phantom: PhantomData,
        }
    }
}

/// Simple command
impl<B, AddressType> CommandOperation<'_, B, AddressType, (), ()>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
    {
        self.block.interface().dispatch_command(
            self.address,
            &mut [],
            &FieldsetMetadata::DEFAULT,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device
    pub fn dispatch_async(
        self,
    ) -> impl Future<Output = Result<(), <B::Interface as CommandInterfaceBase>::Error>>
    where
        B::Interface: AsyncCommandInterface,
    {
        self.block.interface().dispatch_command(
            self.address,
            &mut [],
            &FieldsetMetadata::DEFAULT,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }
}

/// Only input
impl<B, AddressType, InFieldset> CommandOperation<'_, B, AddressType, InFieldset, ()>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
    InFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub fn dispatch(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.block.interface().dispatch_command(
            self.address,
            in_fields.as_slice_mut(),
            &InFieldset::METADATA,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.block
            .interface()
            .dispatch_command(
                self.address,
                in_fields.as_slice_mut(),
                &InFieldset::METADATA,
                &mut [],
                &FieldsetMetadata::DEFAULT,
            )
            .await
    }
}

/// Only output
impl<B, AddressType, OutFieldset> CommandOperation<'_, B, AddressType, (), OutFieldset>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
    OutFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
    {
        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            self.address,
            &mut [],
            &FieldsetMetadata::DEFAULT,
            out_fields.as_slice_mut(),
            &OutFieldset::METADATA,
        )?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
    {
        let mut out_fields = OutFieldset::ZERO;

        self.block
            .interface()
            .dispatch_command(
                self.address,
                &mut [],
                &FieldsetMetadata::DEFAULT,
                out_fields.as_slice_mut(),
                &OutFieldset::METADATA,
            )
            .await?;

        Ok(out_fields)
    }
}

/// Input and output
impl<B, AddressType, InFieldset, OutFieldset>
    CommandOperation<'_, B, AddressType, InFieldset, OutFieldset>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
    InFieldset: Fieldset,
    OutFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub fn dispatch(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            self.address,
            in_fields.as_slice_mut(),
            &InFieldset::METADATA,
            out_fields.as_slice_mut(),
            &OutFieldset::METADATA,
        )?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block
            .interface()
            .dispatch_command(
                self.address,
                in_fields.as_slice_mut(),
                &InFieldset::METADATA,
                out_fields.as_slice_mut(),
                &OutFieldset::METADATA,
            )
            .await?;

        Ok(out_fields)
    }
}
