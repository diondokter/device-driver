use device_driver::CommandInterface;

pub struct DeviceInterface {
    last_command: u8,
    last_input: Vec<u8>,
}

impl CommandInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        input: &[u8],
        output: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.last_command = address;
        self.last_input = input.to_vec();

        let len = output.len().min(input.len());
        output[..len].copy_from_slice(&input[..len]);
        Ok(())
    }
}

device_driver::create_device!(
    device_name: MyTestDevice,
    dsl: {
        config {
            type CommandAddressType = u8;
            type DefaultByteOrder = LE;
        }
        /// A simple command
        command Simple = 0,
        /// A command with only inputs
        command Input {
            const ADDRESS = 1;
            const SIZE_BITS_IN = 16;

            in {
                /// The value!
                val: uint = 0..16,
            }
        },
        /// A command with only outputs
        command Output {
            const ADDRESS = 2;
            const SIZE_BITS_OUT = 8;

            out {
                /// The value!
                val: uint = 0..8,
            }
        },
        /// A command with inputs and outputs
        command InOut {
            const ADDRESS = 3;
            const SIZE_BITS_IN = 16;
            const SIZE_BITS_OUT = 8;

            in {
                /// The value!
                val: uint = 0..16,
            }
            out {
                /// The value!
                val: uint = 0..8,
            }
        },
    }
);

#[test]
fn command_combinations() {
    let mut device = MyTestDevice::new(DeviceInterface {
        last_command: 0xFF,
        last_input: Vec::new(),
    });

    device.simple().dispatch().unwrap();
    assert_eq!(device.interface.last_command, 0);
    assert_eq!(device.interface.last_input, vec![]);

    device.input().dispatch(|reg| reg.set_val(123)).unwrap();
    assert_eq!(device.interface.last_command, 1);
    assert_eq!(device.interface.last_input, vec![0x7B, 0x00]);

    let out = device.output().dispatch().unwrap();
    assert_eq!(device.interface.last_command, 2);
    assert_eq!(device.interface.last_input, vec![]);
    assert_eq!(out.val(), 0);

    let out = device.in_out().dispatch(|reg| reg.set_val(123)).unwrap();
    assert_eq!(device.interface.last_command, 3);
    assert_eq!(device.interface.last_input, vec![0x7B, 0x00]);
    assert_eq!(out.val(), 0x7B);
}
