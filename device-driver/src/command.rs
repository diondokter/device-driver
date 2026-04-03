use core::marker::PhantomData;

use crate::{Fieldset, FieldsetMetadata};

/// A trait to represent the interface to the device.
///
/// This is called to dispatch commands.
pub trait CommandInterface {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

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
pub trait AsyncCommandInterface {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

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
pub struct CommandOperation<'i, Interface, AddressType: Copy, InFieldset, OutFieldset> {
    interface: &'i mut Interface,
    address: AddressType,
    _phantom: PhantomData<(InFieldset, OutFieldset)>,
}

impl<'i, Interface, AddressType: Copy, InFieldset, OutFieldset>
    CommandOperation<'i, Interface, AddressType, InFieldset, OutFieldset>
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

/// Simple command
impl<Interface, AddressType: Copy> CommandOperation<'_, Interface, AddressType, (), ()>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<(), Interface::Error> {
        self.interface.dispatch_command(
            &FieldsetMetadata::DEFAULT,
            &FieldsetMetadata::DEFAULT,
            self.address,
            &[],
            &mut [],
        )
    }
}

/// Only input
impl<Interface, AddressType: Copy, InFieldset: Fieldset>
    CommandOperation<'_, Interface, AddressType, InFieldset, ()>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(self, f: impl FnOnce(&mut InFieldset)) -> Result<(), Interface::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.interface.dispatch_command(
            &InFieldset::METADATA,
            &FieldsetMetadata::DEFAULT,
            self.address,
            in_fields.get_inner_buffer(),
            &mut [],
        )
    }
}

/// Only output
impl<Interface, AddressType: Copy, OutFieldset: Fieldset>
    CommandOperation<'_, Interface, AddressType, (), OutFieldset>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<OutFieldset, Interface::Error> {
        let mut out_fields = OutFieldset::ZERO;

        self.interface.dispatch_command(
            &FieldsetMetadata::DEFAULT,
            &OutFieldset::METADATA,
            self.address,
            &[],
            out_fields.get_inner_buffer_mut(),
        )?;

        Ok(out_fields)
    }
}

/// Input and output
impl<Interface, AddressType: Copy, InFieldset: Fieldset, OutFieldset: Fieldset>
    CommandOperation<'_, Interface, AddressType, InFieldset, OutFieldset>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, Interface::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.interface.dispatch_command(
            &InFieldset::METADATA,
            &OutFieldset::METADATA,
            self.address,
            in_fields.get_inner_buffer(),
            out_fields.get_inner_buffer_mut(),
        )?;

        Ok(out_fields)
    }
}

/// Simple command async
impl<Interface, AddressType: Copy> CommandOperation<'_, Interface, AddressType, (), ()>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<(), Interface::Error> {
        self.interface
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
impl<Interface, AddressType: Copy, InFieldset: Fieldset>
    CommandOperation<'_, Interface, AddressType, InFieldset, ()>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), Interface::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.interface
            .dispatch_command(
                &InFieldset::METADATA,
                &FieldsetMetadata::DEFAULT,
                self.address,
                in_fields.get_inner_buffer(),
                &mut [],
            )
            .await
    }
}

/// Only output async
impl<Interface, AddressType: Copy, OutFieldset: Fieldset>
    CommandOperation<'_, Interface, AddressType, (), OutFieldset>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<OutFieldset, Interface::Error> {
        let mut out_fields = OutFieldset::ZERO;

        self.interface
            .dispatch_command(
                &FieldsetMetadata::DEFAULT,
                &OutFieldset::METADATA,
                self.address,
                &[],
                out_fields.get_inner_buffer_mut(),
            )
            .await?;

        Ok(out_fields)
    }
}

/// Input and output async
impl<Interface, AddressType: Copy, InFieldset: Fieldset, OutFieldset: Fieldset>
    CommandOperation<'_, Interface, AddressType, InFieldset, OutFieldset>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, Interface::Error> {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.interface
            .dispatch_command(
                &InFieldset::METADATA,
                &OutFieldset::METADATA,
                self.address,
                in_fields.get_inner_buffer(),
                out_fields.get_inner_buffer_mut(),
            )
            .await?;

        Ok(out_fields)
    }
}
