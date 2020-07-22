/// General Device trait
pub trait Device<I, P> {
    type Error;

    /// Create a new instance of the device with the given interface
    fn new(interface: I, pins: P) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

#[macro_export]
macro_rules! create_device {
    (
        $device_name:ident {
            $(interface: $interface_bounds:path,)?
            $(pins: $pins_bounds:path,)?
            error: $error_type:ty,
        }
    ) => {
        use device_driver::hl::Device;

        pub trait InterfaceBounds = $($interface_bounds)?;
        pub trait PinsBounds = $($pins_bounds)?;

        pub struct $device_name<I: InterfaceBounds, P: PinsBounds> {
            interface: I,
            pins: P,
        }

        impl<I: InterfaceBounds, P: PinsBounds> Device<I, P> for $device_name<I, P> {
            type Error = $error_type;

            fn new(interface: I, pins: P) -> Result<Self, Self::Error>
            where
                Self: Sized,
            {
                Ok(Self { interface, pins })
            }
        }
    };
}
