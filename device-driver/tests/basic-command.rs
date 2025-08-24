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
        _size_bits_in: u32,
        input: &[u8],
        _size_bits_out: u32,
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
    kdl: {
        device MyTestDevice {
            default-byte-order LE
            command-address-type u8
            /// A simple command
            command Simple {
                address 0
            }
            /// A command with only inputs
            command Input {
                address 1
                in size-bits=16 {
                    /// The value!
                    (uint)val @15:0
                }
            }
            /// A command with only outputs
            command Output {
                address 2
                out size-bits=8 {
                    /// The value!
                    (uint)val @7:0
                }
            }
            /// A command with inputs and outputs
            command InOut {
                address 3
                in size-bits=16 {
                    /// The value!
                    (uint)val @15:0
                }
                out size-bits=8 {
                    /// The value!
                    (uint)val @7:0
                }
            }
        }
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
