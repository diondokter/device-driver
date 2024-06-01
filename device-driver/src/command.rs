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
