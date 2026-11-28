#![cfg_attr(not(feature = "trace"), expect(clippy::let_unit_value))]

use core::marker::PhantomData;

#[cfg(feature = "trace")]
use device_driver_trace::encoder::Event;

use crate::{
    Address, Block, Fieldset, FieldsetMetadata, Index, NotFieldset, NotRepeating, Repeating,
};

#[cfg(feature = "trace")]
type ModifyEvent = Option<Event>;
#[cfg(not(feature = "trace"))]
type ModifyEvent = ();

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
pub struct CommandOperation<'b, B, AddressType, InFieldset, OutFieldset, Repeat>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    block: &'b mut B,
    address: AddressType,
    #[cfg(feature = "trace")]
    object_id: u32,
    _phantom: PhantomData<(InFieldset, OutFieldset, Repeat)>,
}

impl<'d, B, AddressType, InFieldset, OutFieldset, Repeat>
    CommandOperation<'d, B, AddressType, InFieldset, OutFieldset, Repeat>
where
    B: Block,
    B::Interface: CommandInterfaceBase<AddressType = AddressType>,
    AddressType: Address,
{
    #[doc(hidden)]
    pub fn new(block: &'d mut B, address: AddressType, _object_id: u32) -> Self {
        Self {
            block,
            address,
            #[cfg(feature = "trace")]
            object_id: _object_id,
            _phantom: PhantomData,
        }
    }

    fn send_none_event(&mut self, _index: Option<i32>) {
        #[cfg(feature = "trace")]
        if let Some(trace_interface) = crate::global_trace_interface() {
            use device_driver_trace::encoder::Event;

            Event::new(trace_interface, B::SOURCE_CRC).send_final_frame(trace_interface, |f| {
                f.send_timestamp(trace_interface.timestamp());
                f.send_object_identifier(&device_driver_trace::ObjectIdentifier::new(
                    self.object_id,
                    _index,
                ));
                self.address.get_le_bytes(|address| f.send_address(address));
            });
        }
    }

    fn send_single_event(&mut self, _index: Option<i32>, _fs: &impl Fieldset, _write: bool) {
        #[cfg(feature = "trace")]
        if let Some(trace_interface) = crate::global_trace_interface() {
            use device_driver_trace::encoder::Event;

            Event::new(trace_interface, B::SOURCE_CRC).send_final_frame(trace_interface, |f| {
                f.send_timestamp(trace_interface.timestamp());
                f.send_object_identifier(&device_driver_trace::ObjectIdentifier::new(
                    self.object_id,
                    _index,
                ));
                self.address.get_le_bytes(|address| f.send_address(address));
                f.send_bytes(_write, _fs.as_slice());
            });
        }
    }

    fn start_inout_event(&mut self, _index: Option<i32>, _in: &impl Fieldset) -> ModifyEvent {
        #[cfg(feature = "trace")]
        if let Some(trace_interface) = crate::global_trace_interface() {
            Some(
                Event::new(trace_interface, B::SOURCE_CRC).send_frame(trace_interface, |f| {
                    f.send_timestamp(trace_interface.timestamp());
                    f.send_object_identifier(&device_driver_trace::ObjectIdentifier::new(
                        self.object_id,
                        _index,
                    ));
                    self.address.get_le_bytes(|address| f.send_address(address));
                    f.send_bytes(true, _in.as_slice());
                }),
            )
        } else {
            None
        }
    }

    fn finish_inout_event(&mut self, _event: ModifyEvent, _out: &impl Fieldset) {
        #[cfg(feature = "trace")]
        if let Some(event) = _event {
            if let Some(trace_interface) = crate::global_trace_interface() {
                event.send_final_frame(trace_interface, |f| {
                    f.send_bytes(false, _out.as_slice());
                });
            } else {
                event.discard();
            }
        }
    }

    /// Dispatch the command to the device
    #[allow(private_bounds)]
    pub fn dispatch(mut self) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: NotFieldset,
        Repeat: NotRepeating,
    {
        self.send_none_event(None);

        self.block.interface().dispatch_command(
            self.address,
            &mut [],
            &FieldsetMetadata::DEFAULT,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device
    #[allow(private_bounds)]
    #[track_caller]
    pub fn dispatch_at(
        mut self,
        index: Repeat::Index,
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: NotFieldset,
        Repeat: Repeating,
    {
        self.send_none_event(Some(index.index()));

        self.block.interface().dispatch_command(
            Repeat::calc_address(self.address, index),
            &mut [],
            &FieldsetMetadata::DEFAULT,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device
    #[allow(private_bounds)]
    pub fn dispatch_async(
        mut self,
    ) -> impl Future<Output = Result<(), <B::Interface as CommandInterfaceBase>::Error>>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: NotFieldset,
        Repeat: NotRepeating,
    {
        self.send_none_event(None);

        self.block.interface().dispatch_command(
            self.address,
            &mut [],
            &FieldsetMetadata::DEFAULT,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device
    #[allow(private_bounds)]
    pub fn dispatch_at_async(
        mut self,
        index: Repeat::Index,
    ) -> impl Future<Output = Result<(), <B::Interface as CommandInterfaceBase>::Error>>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: NotFieldset,
        Repeat: Repeating,
    {
        self.send_none_event(Some(index.index()));

        self.block.interface().dispatch_command(
            Repeat::calc_address(self.address, index),
            &mut [],
            &FieldsetMetadata::DEFAULT,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device with an input
    #[allow(private_bounds)]
    pub fn dispatch_in(
        mut self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: Fieldset,
        OutFieldset: NotFieldset,
        Repeat: NotRepeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.send_single_event(None, &in_fields, true);

        self.block.interface().dispatch_command(
            self.address,
            in_fields.as_slice_mut(),
            &InFieldset::METADATA,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device with an input
    #[track_caller]
    #[allow(private_bounds)]
    pub fn dispatch_in_at(
        mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: Fieldset,
        OutFieldset: NotFieldset,
        Repeat: Repeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.send_single_event(Some(index.index()), &in_fields, true);

        self.block.interface().dispatch_command(
            Repeat::calc_address(self.address, index),
            in_fields.as_slice_mut(),
            &InFieldset::METADATA,
            &mut [],
            &FieldsetMetadata::DEFAULT,
        )
    }

    /// Dispatch the command to the device with an input
    #[allow(private_bounds)]
    pub async fn dispatch_in_async(
        mut self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: Fieldset,
        OutFieldset: NotFieldset,
        Repeat: NotRepeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.send_single_event(None, &in_fields, true);

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

    /// Dispatch the command to the device with an input
    #[allow(private_bounds)]
    pub async fn dispatch_in_at_async(
        mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<(), <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: Fieldset,
        OutFieldset: NotFieldset,
        Repeat: Repeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        self.send_single_event(Some(index.index()), &in_fields, true);

        self.block
            .interface()
            .dispatch_command(
                Repeat::calc_address(self.address, index),
                in_fields.as_slice_mut(),
                &InFieldset::METADATA,
                &mut [],
                &FieldsetMetadata::DEFAULT,
            )
            .await
    }

    /// Dispatch the command to the device with an output
    #[allow(private_bounds)]
    pub fn dispatch_out(
        mut self,
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: Fieldset,
        Repeat: NotRepeating,
    {
        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            self.address,
            &mut [],
            &FieldsetMetadata::DEFAULT,
            out_fields.as_slice_mut(),
            &OutFieldset::METADATA,
        )?;

        self.send_single_event(None, &out_fields, false);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an output
    #[track_caller]
    #[allow(private_bounds)]
    pub fn dispatch_out_at(
        mut self,
        index: Repeat::Index,
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: Fieldset,
        Repeat: Repeating,
    {
        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            Repeat::calc_address(self.address, index),
            &mut [],
            &FieldsetMetadata::DEFAULT,
            out_fields.as_slice_mut(),
            &OutFieldset::METADATA,
        )?;

        self.send_single_event(Some(index.index()), &out_fields, false);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an output
    #[allow(private_bounds)]
    pub async fn dispatch_out_async(
        mut self,
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: Fieldset,
        Repeat: NotRepeating,
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

        self.send_single_event(None, &out_fields, false);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an output
    #[allow(private_bounds)]
    pub async fn dispatch_out_at_async(
        mut self,
        index: Repeat::Index,
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: NotFieldset,
        OutFieldset: Fieldset,
        Repeat: Repeating,
    {
        let mut out_fields = OutFieldset::ZERO;

        self.block
            .interface()
            .dispatch_command(
                Repeat::calc_address(self.address, index),
                &mut [],
                &FieldsetMetadata::DEFAULT,
                out_fields.as_slice_mut(),
                &OutFieldset::METADATA,
            )
            .await?;

        self.send_single_event(Some(index.index()), &out_fields, false);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an input and output
    pub fn dispatch_inout(
        mut self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: Fieldset,
        OutFieldset: Fieldset,
        Repeat: NotRepeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let event = self.start_inout_event(None, &in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            self.address,
            in_fields.as_slice_mut(),
            &InFieldset::METADATA,
            out_fields.as_slice_mut(),
            &OutFieldset::METADATA,
        )?;

        self.finish_inout_event(event, &out_fields);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an input and output
    #[track_caller]
    pub fn dispatch_inout_at(
        mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: CommandInterface,
        InFieldset: Fieldset,
        OutFieldset: Fieldset,
        Repeat: Repeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let event = self.start_inout_event(None, &in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block.interface().dispatch_command(
            Repeat::calc_address(self.address, index),
            in_fields.as_slice_mut(),
            &InFieldset::METADATA,
            out_fields.as_slice_mut(),
            &OutFieldset::METADATA,
        )?;

        self.finish_inout_event(event, &out_fields);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an input and output
    pub async fn dispatch_inout_async(
        mut self,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: Fieldset,
        OutFieldset: Fieldset,
        Repeat: NotRepeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let event = self.start_inout_event(None, &in_fields);

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

        self.finish_inout_event(event, &out_fields);

        Ok(out_fields)
    }

    /// Dispatch the command to the device with an input and output
    pub async fn dispatch_inout_at_async(
        mut self,
        index: Repeat::Index,
        f: impl FnOnce(&mut InFieldset),
    ) -> Result<OutFieldset, <B::Interface as CommandInterfaceBase>::Error>
    where
        B::Interface: AsyncCommandInterface,
        InFieldset: Fieldset,
        OutFieldset: Fieldset,
        Repeat: Repeating,
    {
        let mut in_fields = InFieldset::ZERO;
        f(&mut in_fields);

        let event = self.start_inout_event(None, &in_fields);

        let mut out_fields = OutFieldset::ZERO;

        self.block
            .interface()
            .dispatch_command(
                Repeat::calc_address(self.address, index),
                in_fields.as_slice_mut(),
                &InFieldset::METADATA,
                out_fields.as_slice_mut(),
                &OutFieldset::METADATA,
            )
            .await?;

        self.finish_inout_event(event, &out_fields);

        Ok(out_fields)
    }
}
