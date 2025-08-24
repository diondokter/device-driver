use device_driver::AsyncRegisterInterface;

pub struct DeviceInterface;

impl AsyncRegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    async fn write_register(
        &mut self,
        _address: Self::AddressType,
        _size_bits: u32,
        _data: &[u8],
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    async fn read_register(
        &mut self,
        _address: Self::AddressType,
        _size_bits: u32,
        _data: &mut [u8],
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

device_driver::create_device!(
    kdl: "
        device MyTestDevice {
            register-address-type u8

            register Foo {
                address 0
                fields size-bits=8 {
                    (bool)value0 @0
                }
            }
        }
    "
);

#[test]
fn async_compiles() {
    let mut device = MyTestDevice::new(DeviceInterface);
    let _future = async { device.foo().read_async().await };
}
