use core::marker::PhantomData;

use crate::FieldSet;

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
        address: Self::AddressType,
        size_bits_in: u32,
        input: &[u8],
        size_bits_out: u32,
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
        address: Self::AddressType,
        size_bits_in: u32,
        input: &[u8],
        size_bits_out: u32,
        output: &mut [u8],
    ) -> Result<(), Self::Error>;
}

/// Intermediate type for doing command operations
pub struct CommandOperation<'i, Interface, AddressType: Copy, InFieldSet, OutFieldSet> {
    interface: &'i mut Interface,
    address: AddressType,
    _phantom: PhantomData<(InFieldSet, OutFieldSet)>,
}

impl<'i, Interface, AddressType: Copy, InFieldSet, OutFieldSet>
    CommandOperation<'i, Interface, AddressType, InFieldSet, OutFieldSet>
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
        self.interface
            .dispatch_command(self.address, 0, &[], 0, &mut [])
    }
}

/// Only input
impl<Interface, AddressType: Copy, InFieldSet: FieldSet>
    CommandOperation<'_, Interface, AddressType, InFieldSet, ()>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(self, f: impl FnOnce(&mut InFieldSet)) -> Result<(), Interface::Error> {
        let mut in_fields = InFieldSet::new_with_zero();
        f(&mut in_fields);

        self.interface.dispatch_command(
            self.address,
            InFieldSet::SIZE_BITS,
            InFieldSet::BUFFER::from(in_fields).as_ref(),
            0,
            &mut [],
        )
    }
}

/// Only output
impl<Interface, AddressType: Copy, OutFieldSet: FieldSet>
    CommandOperation<'_, Interface, AddressType, (), OutFieldSet>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<OutFieldSet, Interface::Error> {
        let mut buffer = OutFieldSet::BUFFER::from(OutFieldSet::new_with_zero());

        self.interface.dispatch_command(
            self.address,
            0,
            &[],
            OutFieldSet::SIZE_BITS,
            buffer.as_mut(),
        )?;

        Ok(buffer.into())
    }
}

/// Input and output
impl<Interface, AddressType: Copy, InFieldSet: FieldSet, OutFieldSet: FieldSet>
    CommandOperation<'_, Interface, AddressType, InFieldSet, OutFieldSet>
where
    Interface: CommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub fn dispatch(
        self,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<OutFieldSet, Interface::Error> {
        let mut in_fields = InFieldSet::new_with_zero();
        f(&mut in_fields);

        let mut buffer = OutFieldSet::BUFFER::from(OutFieldSet::new_with_zero());

        self.interface.dispatch_command(
            self.address,
            InFieldSet::SIZE_BITS,
            InFieldSet::BUFFER::from(in_fields).as_ref(),
            OutFieldSet::SIZE_BITS,
            buffer.as_mut(),
        )?;

        Ok(buffer.into())
    }
}

/// Simple command async
impl<'i, Interface, AddressType: Copy> CommandOperation<'i, Interface, AddressType, (), ()>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<(), Interface::Error> {
        self.interface
            .dispatch_command(self.address, 0, &[], 0, &mut [])
            .await
    }
}

/// Only input async
impl<'i, Interface, AddressType: Copy, InFieldSet: FieldSet>
    CommandOperation<'i, Interface, AddressType, InFieldSet, ()>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<(), Interface::Error> {
        let mut in_fields = InFieldSet::new_with_zero();
        f(&mut in_fields);

        self.interface
            .dispatch_command(
                self.address,
                InFieldSet::SIZE_BITS,
                InFieldSet::BUFFER::from(in_fields).as_ref(),
                0,
                &mut [],
            )
            .await
    }
}

/// Only output async
impl<'i, Interface, AddressType: Copy, OutFieldSet: FieldSet>
    CommandOperation<'i, Interface, AddressType, (), OutFieldSet>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<OutFieldSet, Interface::Error> {
        let mut buffer = OutFieldSet::BUFFER::from(OutFieldSet::new_with_zero());

        self.interface
            .dispatch_command(
                self.address,
                0,
                &[],
                OutFieldSet::SIZE_BITS,
                buffer.as_mut(),
            )
            .await?;

        Ok(buffer.into())
    }
}

/// Input and output async
impl<'i, Interface, AddressType: Copy, InFieldSet: FieldSet, OutFieldSet: FieldSet>
    CommandOperation<'i, Interface, AddressType, InFieldSet, OutFieldSet>
where
    Interface: AsyncCommandInterface<AddressType = AddressType>,
{
    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<OutFieldSet, Interface::Error> {
        let mut in_fields = InFieldSet::new_with_zero();
        f(&mut in_fields);

        let mut buffer = OutFieldSet::BUFFER::from(OutFieldSet::new_with_zero());

        self.interface
            .dispatch_command(
                self.address,
                InFieldSet::SIZE_BITS,
                InFieldSet::BUFFER::from(in_fields).as_ref(),
                OutFieldSet::SIZE_BITS,
                buffer.as_mut(),
            )
            .await?;

        Ok(buffer.into())
    }
}
