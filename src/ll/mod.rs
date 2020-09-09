use crate::ll::register::RegisterError;
use core::fmt::Debug;

//#[macro_use]
//pub mod memory;
#[macro_use]
pub mod register;

/// General Device trait
pub trait LowLevelDevice<I> {
    /// Create a new instance of the device with the given interface
    fn new(interface: I) -> Self
    where
        Self: Sized;

    /// Destruct the device and give back the interface
    fn free(self) -> I;
}

#[macro_export]
macro_rules! create_low_level_device {
    (
        $device_name:ident
    ) => {
        use device_driver::ll::LowLevelDevice;

        pub struct $device_name<I> {
            interface: I,
        }

        impl<I> LowLevelDevice<I> for $device_name<I> {
            fn new(interface: I) -> Self
            where
                Self: Sized,
            {
                Self { interface }
            }

            fn free(self) -> I {
                self.interface
            }
        }
    };
}

#[derive(Debug)]
pub enum LowLevelError<HE: Debug> {
    RegisterError(RegisterError<HE>),
}

impl<HE: Debug> From<RegisterError<HE>> for LowLevelError<HE> {
    fn from(val: RegisterError<HE>) -> Self {
        LowLevelError::RegisterError(val)
    }
}
