use device_driver::{FieldsetMetadata, RegisterInterface, RegisterInterfaceBase};

pub struct DeviceInterface;

impl RegisterInterfaceBase for DeviceInterface {
    type Error = core::convert::Infallible;
    type AddressType = u8;
}
impl RegisterInterface for DeviceInterface {
    fn write_register(
        &mut self,
        _address: Self::AddressType,
        _data: &mut [u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        todo!()
    }

    fn read_register(
        &mut self,
        _address: Self::AddressType,
        _data: &mut [u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        todo!()
    }
}

device_driver::compile!(
    ddsl: "
        device MyTestDevice {
            register-address-type: u8,

            register FooLe {
                address: 0,
                reset: 0xFF,
                fields: fieldset FooLeFields {
                    byte-order: LE,
                    size-bytes: 32,

                    field value-start 7:0 -> uint,
                    field value-end 255:248 -> uint,
                }
            },
            register FooBe {
                address: 1,
                reset: 0xFF,
                fields: fieldset FooBeFields {
                    byte-order: BE,
                    size-bytes: 32,

                    field value-start 7:0 -> uint,
                    field value-end 255:248 -> uint,
                }
            }
        }
    "
);

#[test]
fn reset_int_properly_extended() {
    let mut device = MyTestDevice::new(DeviceInterface);

    assert_eq!(device.foo_le().reset_value().value_start(), 0xFF);
    assert_eq!(device.foo_be().reset_value().value_start(), 0xFF);

    assert_eq!(device.foo_le().reset_value().value_end(), 0x00);
    assert_eq!(device.foo_be().reset_value().value_end(), 0x00);
}
