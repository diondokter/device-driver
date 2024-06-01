/// A trait to represent the interface to the device.
///
/// This is called to dispatch commands.
pub trait CommandDevice {
    /// The error type
    type Error;

    /// Dispatch a command on the device by sending the command.
    fn dispatch_command(&mut self, id: u32) -> Result<(), Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to asynchronously dispatch commands.
pub trait AsyncCommandDevice {
    /// The error type
    type Error;

    /// Dispatch a command on the device by sending the command.
    async fn dispatch_command(&mut self, id: u32) -> Result<(), Self::Error>;
}

/// Intermediate type for doing command operations
pub struct CommandOperation<'a, D> {
    device: &'a mut D,
    id: u32,
}

impl<'a, D> CommandOperation<'a, D> {
    #[doc(hidden)]
    pub fn new(device: &'a mut D, id: u32) -> Self {
        Self { device, id }
    }
}

impl<'a, D: CommandDevice> CommandOperation<'a, D> {
    /// Dispatch the command to the device
    pub fn dispatch(self) -> Result<(), D::Error> {
        self.device.dispatch_command(self.id)
    }
}

impl<'a, D: AsyncCommandDevice> CommandOperation<'a, D> {
    /// Dispatch the command to the device
    pub async fn dispatch_async(self) -> Result<(), D::Error> {
        self.device.dispatch_command(self.id).await
    }
}
