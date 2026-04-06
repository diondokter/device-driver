use core::marker::PhantomData;

use crate::{Address, Block, Fieldset, FieldsetMetadata};

/// Common properties shared by [CommandInterface] & [AsyncCommandInterface]
pub trait CommandInterfaceBase {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;
}

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
        _metadata_input: &FieldsetMetadata,
        _metadata_output: &FieldsetMetadata,
        address: Self::AddressType,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Self::Error>;
}

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
        _metadata_input: &FieldsetMetadata,
        _metadata_output: &FieldsetMetadata,
        address: Self::AddressType,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Self::Error>;
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
    B::Interface: CommandInterface<AddressType = AddressType>,
    AddressType: Address,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<(), <B::Interface as CommandInterfaceBase>::Error> {
        self.block.interface().dispatch_command(
            &FieldsetMetadata::DEFAULT,
            &FieldsetMetadata::DEFAULT,
            self.address,
            &[],
            &mut [],
        )
    }
}

/// Only input
impl<B, AddressType, InFieldset> CommandOperation<'_, B, AddressType, InFieldset, ()>
where
    B: Block,
    B::Interface: CommandInterface<AddressType = AddressType>,
    AddressType: Address,
    InFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub fn dispatch(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.block.interface().dispatch_command(
            &InFieldset::METADATA,
            &FieldsetMetadata::DEFAULT,
            self.address,
            in_fields.as_slice(),
            &mut [],
        )
    }
}

/// Only output
impl<B, AddressType, OutFieldset> CommandOperation<'_, B, AddressType, (), OutFieldset>
where
    B: Block,
    B::Interface: CommandInterface<AddressType = AddressType>,
    AddressType: Address,
    OutFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error> {
        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            &FieldsetMetadata::DEFAULT,
            &OutFieldset::METADATA,
            self.address,
            &[],
            out_fields.as_slice_mut(),
        )?;

        Ok(out_fields)
    }
}

/// Input and output
impl<B, AddressType, InFieldset, OutFieldset>
    CommandOperation<'_, B, AddressType, InFieldset, OutFieldset>
where
    B: Block,
    B::Interface: CommandInterface<AddressType = AddressType>,
    AddressType: Address,
    InFieldset: Fieldset,
    OutFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub fn dispatch(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            &InFieldset::METADATA,
            &OutFieldset::METADATA,
            self.address,
            in_fields.as_slice(),
            out_fields.as_slice_mut(),
        )?;

        Ok(out_fields)
    }
}

/// Simple command async
impl<B, AddressType> CommandOperation<'_, B, AddressType, (), ()>
where
    B: Block,
    B::Interface: AsyncCommandInterface<AddressType = AddressType>,
    AddressType: Address,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<(), <B::Interface as CommandInterfaceBase>::Error> {
        self.block
            .interface()
            .dispatch_command(
                &FieldsetMetadata::DEFAULT,
                &FieldsetMetadata::DEFAULT,
                self.address,
                &[],
                &mut [],
            )
            .await
    }
}

/// Only input async
impl<B, AddressType, InFieldset> CommandOperation<'_, B, AddressType, InFieldset, ()>
where
    B: Block,
    B::Interface: AsyncCommandInterface<AddressType = AddressType>,
    AddressType: Address,
    InFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.block
            .interface()
            .dispatch_command(
                &InFieldset::METADATA,
                &FieldsetMetadata::DEFAULT,
                self.address,
                in_fields.as_slice(),
                &mut [],
            )
            .await
    }
}

/// Only output async
impl<B, AddressType, OutFieldset> CommandOperation<'_, B, AddressType, (), OutFieldset>
where
    B: Block,
    B::Interface: AsyncCommandInterface<AddressType = AddressType>,
    AddressType: Address,
    OutFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error> {
        let mut out_fields = OutFieldset::ZERO;

        self.block
            .interface()
            .dispatch_command(
                &FieldsetMetadata::DEFAULT,
                &OutFieldset::METADATA,
                self.address,
                &[],
                out_fields.as_slice_mut(),
            )
            .await?;

        Ok(out_fields)
    }
}

/// Input and output async
impl<B, AddressType, InFieldset, OutFieldset>
    CommandOperation<'_, B, AddressType, InFieldset, OutFieldset>
where
    B: Block,
    B::Interface: AsyncCommandInterface<AddressType = AddressType>,
    AddressType: Address,
    InFieldset: Fieldset,
    OutFieldset: Fieldset,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block
            .interface()
            .dispatch_command(
                &InFieldset::METADATA,
                &OutFieldset::METADATA,
                self.address,
                in_fields.as_slice(),
                out_fields.as_slice_mut(),
            )
            .await?;

        Ok(out_fields)
    }
}
