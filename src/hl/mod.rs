/// General Device trait
pub trait Device<I> {
    type Error;

    /// Create a new instance of the device with the given interface
    fn new(interface: I) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[macro_export]
macro_rules! create_device {
    (
        $device_name:ident {
            error = $error_type:ty,
        }
    ) => {
        use device_driver::hl::Device;

        pub struct $device_name<I> {
            interface: I,
        }

        impl<I> Device<I> for $device_name<I> {
            type Error = $error_type;

            fn new(interface: I) -> Result<Self, Self::Error>
            where
                Self: Sized,
            {
                Ok(Self { interface })
            }
        }
    };
}
