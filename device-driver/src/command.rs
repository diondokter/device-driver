/// A trait to represent the interface to the device.
///
/// This is called to dispatch commands.
pub trait CommandDevice {
    /// The error type
    type Error;
    /// The inner type of the command (that which is sent over the wire to the device)
    type RawType;

    /// Dispatch a command on the device by sending the command.
    fn dispatch_command(&mut self, raw_command: Self::RawType) -> Result<(), Self::Error>;
}

/// A trait to represent the interface to the device.
///
/// This is called to asynchronously dispatch commands.
pub trait AsyncCommandDevice {
    /// The error type
    type Error;
    /// The inner type of the command (that which is sent over the wire to the device)
    type RawType;

    /// Dispatch a command on the device by sending the command.
    async fn dispatch_command(&mut self, raw_command: Self::RawType) -> Result<(), Self::Error>;
}
