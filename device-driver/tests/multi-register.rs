use device_driver::{Block, FieldsetMetadata, RegisterInterface, RegisterInterfaceBase};

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
        address: Self::AddressType,
        data: &[u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &FieldsetMetadata,
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
            },
            register Indexed[IndexEnum*3] {
                address: 16,
                fields: FooFields,
            },
            enum IndexEnum {
                A: _,
                B: _,
            }
        }
    "
);

#[test]
fn multi() {
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
fn array() {
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

    device
        .foo_repeated()
        .modify_array_at(0, |[foo0, _, _, foo3]| {
            foo0.set_value(100);
            foo3.set_value(101);
        })
        .unwrap();

    let [foo0, foo1, foo2, foo3] = device.foo_repeated().read_array_at(0).unwrap();
    assert_eq!(foo0.value(), 100);
    assert_eq!(foo1.value(), 1);
    assert_eq!(foo2.value(), 2);
    assert_eq!(foo3.value(), 101);
}

#[test]
fn multi_array() {
    use device_driver::Block;
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .multi_write()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .with(|d| d.foo_repeated().plan_at(0))
        .with(|d| d.foo_repeated().plan_array_at(1))
        .execute(|(_, _, foo0, [foo1, foo2])| {
            foo0.set_value(42);
            foo1.set_value(43);
            foo2.set_value(44);
        })
        .unwrap();

    let (_, _, foo0, [foo1, foo2]) = device
        .multi_read()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .with(|d| d.foo_repeated().plan_at(0))
        .with(|d| d.foo_repeated().plan_array_at(1))
        .execute()
        .unwrap();

    assert_eq!(foo0.value(), 42);
    assert_eq!(foo1.value(), 43);
    assert_eq!(foo2.value(), 44);

    device
        .multi_modify()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .with(|d| d.foo_repeated().plan_at(0))
        .with(|d| d.foo_repeated().plan_array_at(1))
        .execute(|(foo, bar, foo0, [foo1, foo2])| {
            foo.set_value(foo.value() + 1);
            bar.set_value(bar.value() + 1);
            foo0.set_value(foo0.value() + 1);
            foo1.set_value(foo1.value() + 1);
            foo2.set_value(foo2.value() + 1);
        })
        .unwrap();

    let (foo, bar, foo0, [foo1, foo2]) = device
        .multi_read()
        .with(|d| d.foo().plan())
        .with(|d| d.bar().plan())
        .with(|d| d.foo_repeated().plan_at(0))
        .with(|d| d.foo_repeated().plan_array_at(1))
        .execute()
        .unwrap();

    assert_eq!(foo.value(), 1);
    assert_eq!(bar.value(), 1);
    assert_eq!(foo0.value(), 43);
    assert_eq!(foo1.value(), 44);
    assert_eq!(foo2.value(), 45);
}

#[test]
#[should_panic = "array too long. Requested 3, max len remaining at requested index is 2"]
fn array_oob() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device.foo_repeated().read_array_at::<3>(2).unwrap();
}

#[test]
#[should_panic = "array too long. Requested 3, max len remaining at requested index is 2"]
fn plan_array_oob() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device
        .multi_read()
        .with(|d| d.foo_repeated().plan_array_at::<3>(2));
}
