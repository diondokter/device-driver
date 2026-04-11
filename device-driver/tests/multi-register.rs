use device_driver::{FieldsetMetadata, RegisterInterface, RegisterInterfaceBase};

pub struct DeviceInterface {
    device_memory: [u8; 128],
}

impl Default for DeviceInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceInterface {
    pub const fn new() -> Self {
        Self {
            device_memory: [0; 128],
        }
    }
}

impl RegisterInterfaceBase for DeviceInterface {
    type Error = ();
    type AddressType = u8;
}
impl RegisterInterface for DeviceInterface {
    fn write_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        _metadata: &FieldsetMetadata,
        address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::compile!(
    ddsl: "
        device MyTestDevice {
            byte-order: LE,
            register-address-type: u8,
            register-address-mode: mapped,

            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bytes: 3,
                    field value 23:0 -> uint,
                }
            },
            register Bar {
                address: 3,
                fields: fieldset BarFields {
                    size-bytes: 1,
                    field value 7:0 -> uint,
                }
            },
            register FooRepeated[4*3] {
                address: 4,
                fields: FooFields,
            }
        }
    "
);

#[test]
fn multi_test() {
    use device_driver::Block;
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .multi_write()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .execute(|(foo, bar)| {
            foo.set_value(42);
            bar.set_value(42);
        })
        .unwrap();

    let (foo, bar) = device
        .multi_read()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .execute()
        .unwrap();

    assert_eq!(foo.value(), 42);
    assert_eq!(bar.value(), 42);

    device
        .multi_modify()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .execute(|(foo, bar)| {
            foo.set_value(foo.value() + 1);
            bar.set_value(bar.value() + 1);
        })
        .unwrap();

    let (foo, bar) = device
        .multi_read()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .execute()
        .unwrap();

    assert_eq!(foo.value(), 43);
    assert_eq!(bar.value(), 43);
}

#[test]
fn test_array() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .foo_repeated()
        .modify_array_at(1, |[foo1, foo2]| {
            foo1.set_value(1);
            foo2.set_value(2);
        })
        .unwrap();

    let [foo0, foo1, foo2, foo3] = device.foo_repeated().read_array_at(0).unwrap();
    assert_eq!(foo0.value(), 0);
    assert_eq!(foo1.value(), 1);
    assert_eq!(foo2.value(), 2);
    assert_eq!(foo3.value(), 0);
}
