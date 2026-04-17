use device_driver::{AsyncRegisterInterface, FieldsetMetadata, RegisterInterfaceBase};

pub struct DeviceInterface;

impl RegisterInterfaceBase for DeviceInterface {
    type Error = ();
    type AddressType = u8;
}

impl AsyncRegisterInterface for DeviceInterface {
    async fn write_register(
        &mut self,
        _address: Self::AddressType,
        _data: &[u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }

    async fn read_register(
        &mut self,
        _address: Self::AddressType,
        _data: &mut [u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

device_driver::compile!(
    ddsl: "
        device MyTestDevice {
            register-address-type: u8,

            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bytes: 1,
                    field value0 0 -> bool
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
