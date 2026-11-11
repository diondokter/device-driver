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
        _data: &mut [u8],
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
    unstable_ddsl: "
        device MyTestDevice {
            register-address-type: u8,
            default-access: RW,

            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bytes: 1,
                    field value0 0 -> bool
                }
            },
            register FooRepeated [4 stride 1] {
                address: 1,
                fields: FooFields,
            }
        }
    "
);

#[test]
fn async_compiles() {
    let mut device = MyTestDevice::new(DeviceInterface);
    let _future = async { device.foo().read_async().await };
}

#[test]
fn async_pass_through_compiles() {
    // Compiles if we are able to construct pass-through futures
    fn foo_read_async_pass_through(
        device: &mut MyTestDevice<DeviceInterface>,
    ) -> impl Future<Output = Result<FooFields, ()>> {
        device.foo().read_async()
    }

    let mut device = MyTestDevice::new(DeviceInterface);
    let _future = foo_read_async_pass_through(&mut device);
}

#[test]
fn async_repeated_ops_compiles() {
    let mut device = MyTestDevice::new(DeviceInterface);

    // Relies on this device NOT having register-address-mode set.
    let _future = async {
        let _ = device.foo_repeated().read_at_async(1).await;
        let _ = device.foo_repeated().write_at_async(1, |_| {}).await;
        let _ = device
            .foo_repeated()
            .write_with_zero_at_async(1, |_| {})
            .await;
        let _ = device.foo_repeated().modify_at_async(1, |_| {}).await;
    };
}
