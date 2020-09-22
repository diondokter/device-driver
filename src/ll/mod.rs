//#[macro_use]
//pub mod memory;

/// Module that helps with creating a register interface
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
        $(#[$device_doc:meta])*
        $device_name:ident {
            errors: [$($error_type:ident),*],
            hardware_interface_requirements: {$($hardware_interface_bound_type:tt)*},
            hardware_interface_capabilities: $hardware_interface_capabilities:tt $(,)?
        }
    ) => {
        use device_driver::ll::LowLevelDevice;
        use device_driver::ll::register::ConversionError;

        /// Marker trait for hardware interface implementations
        pub trait HardwareInterface : $($hardware_interface_bound_type)* $hardware_interface_capabilities

        $(#[$device_doc])*
        pub struct $device_name<I: HardwareInterface> {
            interface: I,
        }

        impl<I: HardwareInterface> $device_name<I> {
            pub fn interface(&mut self) -> &mut I {
                &mut self.interface
            }
        }

        impl<I: HardwareInterface> LowLevelDevice<I> for $device_name<I> {
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
            $($error_type($error_type))*
        }

        impl From<ConversionError> for LowLevelError {
            fn from(_: ConversionError) -> Self {
                LowLevelError::ConversionError
            }
        }

        $(
        impl From<$error_type> for LowLevelError {
            fn from(val: $error_type) -> Self {
                LowLevelError::$error_type(val)
            }
        }
        )*
    };
}
