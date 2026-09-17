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
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        assert_eq!(data.len(), 3);
        println!("{address}");
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        data: &mut [u8],
        _metadata: &FieldsetMetadata,
    ) -> Result<(), Self::Error> {
        assert_eq!(data.len(), 3);
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::compile!(
    options: "--rust-defmt-feature=defmt",
    unstable_ddsl: "
        device MyTestDevice {
            register-address-type: u8,
            default-access: RW,

            register OutputPort {
                address: 0x00, 
                reset: 0b0101_0101, 
                fields: fieldset _ {  
                    size-bytes: 1,
                    field O[8 stride 1] 0,  
                }
            },
        }
    "
);

#[test]
fn field_repeats() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    let reset = device.output_port().reset_value();
    println!("reset value: {reset:?}, {:b}", reset.bits[0]);

    for i in 0..8 {
        let read_value = reset.o(i);
        let expected_value = i.is_multiple_of(2);
        println!("{i} - read: {read_value}, expected: {expected_value}");
        assert_eq!(read_value, expected_value, "{i}");
    }
}
