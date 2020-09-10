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
    ({
        name: $device_name:ident,
        $(errors: [$($error_type:ident),*], $(,)?)?
    }) => {
        use device_driver::ll::LowLevelDevice;
        use device_driver::ll::register::ConversionError;

        pub(crate) struct $device_name<I> {
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

        #[derive(Debug)]
        pub enum LowLevelError {
            ConversionError,
            $($($error_type($error_type))*)*
        }

        impl From<ConversionError> for LowLevelError {
            fn from(_: ConversionError) -> Self {
                LowLevelError::ConversionError
            }
        }

        $($(
        impl From<$error_type> for LowLevelError {
            fn from(val: $error_type) -> Self {
                LowLevelError::$error_type(val)
            }
        }
        )*)*
    };
}
