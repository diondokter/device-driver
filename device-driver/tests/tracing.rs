use device_driver::{Block, FieldsetMetadata, RegisterInterface, RegisterInterfaceBase};
use device_driver_trace::{ObjectIdentifier, decoder::{Event, Operation}};

pub struct DeviceInterface {
    device_memory: [u8; 128],
    trace: Vec<u8>,
    trace_id: u8,
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
            trace: Vec::new(),
            trace_id: 0,
        }
    }
}

impl RegisterInterfaceBase for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn trace_interface(&mut self) -> Option<&mut impl device_driver::TraceInterface> {
        Some(self)
    }
}
impl RegisterInterface for DeviceInterface {
    fn write_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
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

impl device_driver::TraceInterface for DeviceInterface {
    fn instance(&mut self) -> u8 {
        0
    }

    fn write(&mut self, byte: u8) {
        self.trace.push(byte);
    }

    fn timestamp(&mut self) -> Option<u64> {
        Some(42)
    }

    fn id(&mut self) -> u8 {
        self.trace_id = self.trace_id.wrapping_add(1);
        self.trace_id
    }
}

device_driver::compile!(
    unstable_ddsl: "
        device MyTestDevice {
            register-address-type: u8,
            default-access: RW,

            register SingleByte {
                address: 0,
                fields: fieldset FooFields {
                    size-bytes: 1,

                    field value 7:0,
                }
            },
        }
    "
);

#[test]
fn test_single_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device.single_byte().write(|reg| reg.set_value(0xDD)).unwrap();

    let mut decoder = device_driver_trace::decoder::StreamingDecoder::new();
    let events = decoder.push_bytes(&device.interface().trace);

    assert_eq!(
        events,
        vec![Ok(Event {
            instance: 0,
            id: 1,
            timestamp: Some(42),
            operations: vec![Operation {
                objects: vec![ObjectIdentifier::new(0x03000000, None)],
                address: vec![0],
                read_data: vec![],
                write_data: vec![0xDD],
            }]
        })]
    )
}
