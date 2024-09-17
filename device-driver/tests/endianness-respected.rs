use device_driver::RegisterInterface;

pub struct DeviceInterface;

impl RegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    #[track_caller]
    fn write_register(
        &mut self,
        _address: Self::AddressType,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        // Assert data is little endian
        assert_eq!(data, &[0x34, 0x12]);

        Ok(())
    }

    fn read_register(
        &mut self,
        _address: Self::AddressType,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        data.copy_from_slice(&[0x32, 0x12]);
        Ok(())
    }
}

device_driver_macros::implement_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
        }
        register FooLE {
            const ADDRESS = 0;
            const SIZE_BITS = 16;
            type ByteOrder = LE;
            const ALLOW_ADDRESS_OVERLAP = true;
            const RESET_VALUE = 0x1234;

            val: uint = 0..16,
        },
        register FooLEArray {
            const ADDRESS = 0;
            const SIZE_BITS = 16;
            type ByteOrder = LE;
            const ALLOW_ADDRESS_OVERLAP = true;
            const RESET_VALUE = [0x34, 0x12]; // As the read and write functions see it

            val: uint = 0..16,
        },
        register FooBE {
            const ADDRESS = 0;
            const SIZE_BITS = 16;
            type ByteOrder = BE;
            const ALLOW_ADDRESS_OVERLAP = true;
            const RESET_VALUE = 0x3412;

            val: uint = 0..16,
        },
        register FooBEArray {
            const ADDRESS = 0;
            const SIZE_BITS = 16;
            type ByteOrder = BE;
            const ALLOW_ADDRESS_OVERLAP = true;
            const RESET_VALUE = [0x34, 0x12];

            val: uint = 0..16,
        }
    }
);

#[test]
fn little_endian_respected() {
    let mut device = MyTestDevice::new(DeviceInterface);

    device.foo_le().write(|w| w).unwrap();
    device
        .foo_le()
        .write_with_zero(|w| w.set_val(0x1234))
        .unwrap();

    device.foo_le_array().write(|w| w).unwrap();
    device
        .foo_le_array()
        .write_with_zero(|w| w.set_val(0x1234))
        .unwrap();
}

#[test]
fn big_endian_respected() {
    let mut device = MyTestDevice::new(DeviceInterface);

    device.foo_be().write(|w| w).unwrap();
    device
        .foo_be()
        .write_with_zero(|w| w.set_val(0x3412))
        .unwrap();

    device.foo_be_array().write(|w| w).unwrap();
    device
        .foo_be_array()
        .write_with_zero(|w| w.set_val(0x3412))
        .unwrap();
}