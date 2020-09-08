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
        }
    };
}
