use std::any::TypeId;

use device_driver::RegisterInterface;

pub struct DeviceInterface {
    device_memory: [u8; 128],
}

impl DeviceInterface {
    pub const fn new() -> Self {
        Self {
            device_memory: [0; 128],
        }
    }
}

impl RegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn write_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type DefaultByteOrder = LE;
        }
        /// This is the Foo register
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 24;

            val: int = 0..24,
        },
        /// This is the Foo ref
        ref FooRef = register Foo {
            const ADDRESS = 3;
        }
    }
);

#[test]
fn ref_uses_own_address() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device.foo().write(|reg| reg.set_val(123)).unwrap();
    let foo_ref = device.foo_ref().read().unwrap();

    assert_eq!(foo_ref.val(), 0);
}

#[test]
fn refs_have_same_type() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    let foo = device.foo().read().unwrap();
    let foo_ref = device.foo_ref().read().unwrap();

    assert_eq!(type_id_of(&foo), type_id_of(&foo_ref));
}

fn type_id_of<T: 'static + ?Sized>(_: &T) -> TypeId {
    TypeId::of::<T>()
}
