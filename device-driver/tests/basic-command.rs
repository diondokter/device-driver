use device_driver::CommandInterface;

pub struct DeviceInterface;

impl CommandInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn dispatch_command(
        &mut self,
        _address: Self::AddressType,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Self::Error> {
        let out_len = output.len();
        output[..input.len()].copy_from_slice(&input[..out_len]);
        Ok(())
    }
}

pub mod registers {
    use super::*;

    device_driver_macros::implement_device!(
        device_name: MyTestDevice,
        dsl: {
            config {
                type CommandAddressType = u8;
                type DefaultByteOrder = LE;
            }
            command Foo = 0,
        }
    );

    #[test]
    fn feature() {
        let mut device = MyTestDevice::new(DeviceInterface);
        device.foo().dispatch().unwrap();
        todo!()
    }
}
