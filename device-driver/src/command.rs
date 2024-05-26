/// A trait to represent the interface to the device.
///
/// This is called to dispatch commands.
pub trait CommandDevice {
    /// The error type
    type Error;
    /// The inner type of the command (that which is sent over the wire to the device)
    type RawType;

    /// Dispatch a command on the device by sending the command.
    fn dispatch_command<C>(&mut self, command: C) -> Result<(), Self::Error>
    where
        C: Command<RawType = Self::RawType>;
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
    async fn dispatch_command<C>(&mut self, command: C) -> Result<(), Self::Error>
    where
        C: Command<RawType = Self::RawType>;
}

/// The abstraction and description of a command.
///
/// This is meant to be implemented by the macros.
pub trait Command {
    /// The inner type of the command (that which is sent over the wire to the device)
    type RawType;

    /// Get the raw value to send over the wire to the device
    fn get_raw(&self) -> Self::RawType;
}
