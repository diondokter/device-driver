use core::marker::PhantomData;

use crate::{Address, FieldSet, NotRepeating, Repeating};

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
pub struct CommandOperation<'i, Interface, AddressType: Copy, InFieldSet, OutFieldSet, Form> {
    interface: &'i mut Interface,
    address: AddressType,
    _phantom: PhantomData<(InFieldSet, OutFieldSet, Form)>,
}

impl<'i, Interface, AddressType: Copy, InFieldSet, OutFieldSet, Form>
    CommandOperation<'i, Interface, AddressType, InFieldSet, OutFieldSet, Form>
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
impl<Interface, AddressType, Form> CommandOperation<'_, Interface, AddressType, (), (), Form>
where
    AddressType: Address,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<(), Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        self.interface
            .dispatch_command(self.address, 0, &[], 0, &mut [])
    }

    /// Dispatch the command to the device
    pub fn dispatch_at(self, index: Form::Index) -> Result<(), Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        self.interface
            .dispatch_command(Form::calc_address(self.address, index), 0, &[], 0, &mut [])
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<(), Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        self.interface
            .dispatch_command(self.address, 0, &[], 0, &mut [])
            .await
    }

    /// Dispatch the command to the device
    pub async fn dispatch_at_async(self, index: Form::Index) -> Result<(), Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        self.interface
            .dispatch_command(Form::calc_address(self.address, index), 0, &[], 0, &mut [])
            .await
    }
}

/// Only input
impl<Interface, AddressType, InFieldSet, Form>
    CommandOperation<'_, Interface, AddressType, InFieldSet, (), Form>
where
    AddressType: Address,
    InFieldSet: FieldSet,
{
    /// Dispatch the command to the device
    pub fn dispatch(self, f: impl FnOnce(&mut InFieldSet)) -> Result<(), Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        self.interface.dispatch_command(
            self.address,
            InFieldSet::SIZE_BITS,
            in_fields.get_inner_buffer(),
            0,
            &mut [],
        )
    }

    /// Dispatch the command to the device
    pub fn dispatch_at(
        self,
        index: Form::Index,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<(), Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        self.interface.dispatch_command(
            Form::calc_address(self.address, index),
            InFieldSet::SIZE_BITS,
            in_fields.get_inner_buffer(),
            0,
            &mut [],
        )
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<(), Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        self.interface
            .dispatch_command(
                self.address,
                InFieldSet::SIZE_BITS,
                in_fields.get_inner_buffer(),
                0,
                &mut [],
            )
            .await
    }

    /// Dispatch the command to the device
    pub async fn dispatch_at_async(
        self,
        index: Form::Index,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<(), Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        self.interface
            .dispatch_command(
                Form::calc_address(self.address, index),
                InFieldSet::SIZE_BITS,
                in_fields.get_inner_buffer(),
                0,
                &mut [],
            )
            .await
    }
}

/// Only output
impl<Interface, AddressType, OutFieldSet, Form>
    CommandOperation<'_, Interface, AddressType, (), OutFieldSet, Form>
where
    AddressType: Address,
    OutFieldSet: FieldSet,
{
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut out_fields = OutFieldSet::default();

        self.interface.dispatch_command(
            self.address,
            0,
            &[],
            OutFieldSet::SIZE_BITS,
            out_fields.get_inner_buffer_mut(),
        )?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub fn dispatch_at(self, index: Form::Index) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut out_fields = OutFieldSet::default();

        self.interface.dispatch_command(
            Form::calc_address(self.address, index),
            0,
            &[],
            OutFieldSet::SIZE_BITS,
            out_fields.get_inner_buffer_mut(),
        )?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut out_fields = OutFieldSet::default();

        self.interface
            .dispatch_command(
                self.address,
                0,
                &[],
                OutFieldSet::SIZE_BITS,
                out_fields.get_inner_buffer_mut(),
            )
            .await?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub async fn dispatch_at_async(
        self,
        index: Form::Index,
    ) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut out_fields = OutFieldSet::default();

        self.interface
            .dispatch_command(
                Form::calc_address(self.address, index),
                0,
                &[],
                OutFieldSet::SIZE_BITS,
                out_fields.get_inner_buffer_mut(),
            )
            .await?;

        Ok(out_fields)
    }
}

/// Input and output
impl<Interface, AddressType, InFieldSet, OutFieldSet, Form>
    CommandOperation<'_, Interface, AddressType, InFieldSet, OutFieldSet, Form>
where
    AddressType: Address,
    InFieldSet: FieldSet,
    OutFieldSet: FieldSet,
{
    /// Dispatch the command to the device
    pub fn dispatch(self, f: impl FnOnce(&mut InFieldSet)) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        let mut out_fields = OutFieldSet::default();

        self.interface.dispatch_command(
            self.address,
            InFieldSet::SIZE_BITS,
            in_fields.get_inner_buffer(),
            OutFieldSet::SIZE_BITS,
            out_fields.get_inner_buffer_mut(),
        )?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub fn dispatch_at(
        self,
        index: Form::Index,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: CommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        let mut out_fields = OutFieldSet::default();

        self.interface.dispatch_command(
            Form::calc_address(self.address, index),
            InFieldSet::SIZE_BITS,
            in_fields.get_inner_buffer(),
            OutFieldSet::SIZE_BITS,
            out_fields.get_inner_buffer_mut(),
        )?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub async fn dispatch_async(
        self,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: NotRepeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        let mut out_fields = OutFieldSet::default();

        self.interface
            .dispatch_command(
                self.address,
                InFieldSet::SIZE_BITS,
                in_fields.get_inner_buffer(),
                OutFieldSet::SIZE_BITS,
                out_fields.get_inner_buffer_mut(),
            )
            .await?;

        Ok(out_fields)
    }

    /// Dispatch the command to the device
    pub async fn dispatch_at_async(
        self,
        index: Form::Index,
        f: impl FnOnce(&mut InFieldSet),
    ) -> Result<OutFieldSet, Interface::Error>
    where
        Interface: AsyncCommandInterface<AddressType = AddressType>,
        Form: Repeating,
    {
        let mut in_fields = InFieldSet::default();
        f(&mut in_fields);

        let mut out_fields = OutFieldSet::default();

        self.interface
            .dispatch_command(
                Form::calc_address(self.address, index),
                InFieldSet::SIZE_BITS,
                in_fields.get_inner_buffer(),
                OutFieldSet::SIZE_BITS,
                out_fields.get_inner_buffer_mut(),
            )
            .await?;

        Ok(out_fields)
    }
}
