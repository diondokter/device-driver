use device_driver::{CommandInterface, FieldsetMetadata};

pub struct DeviceInterface {
    last_command: u8,
    last_input: Vec<u8>,
}

impl CommandInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn dispatch_command(
        &mut self,
        _metadata_input: &FieldsetMetadata,
        _metadata_output: &FieldsetMetadata,
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

device_driver::compile!(
    ddsl: "
        device MyTestDevice {
            byte-order: LE,
            command-address-type: u8,
            /// A simple command
            command Simple {
                address: 0
            },
            /// A command with only inputs
            command Input {
                address: 1,
                fields-in: fieldset InputFieldsIn {
                    size-bytes: 2,

                    /// The value!
                    field val 15:0 -> uint
                }
            },
            /// A command with only outputs
            command Output {
                address: 2,
                fields-out: fieldset OutputFieldsOut {
                    size-bytes: 1,

                    /// The value!
                    field val 7:0 -> uint
                }
            },
            /// A command with inputs and outputs
            command InOut {
                address: 3,
                fields-in: fieldset InOutFieldsIn {
                    size-bytes: 2,

                    /// The value!
                    field val 15:0 -> uint,
                },
                fields-out: fieldset InOutFieldsOut {
                    size-bytes: 1,

                    /// The value!
                    field val 7:0 -> uint
                }
            }
        }
    "
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
