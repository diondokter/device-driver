use device_driver::RegisterInterface;

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

impl RegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn write_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        assert_eq!(size_bits, 24);
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        assert_eq!(size_bits, 24);
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::create_device!(
    manifest: "tests/basic-register.ddsl"
);

#[test]
fn test_basic_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device.foo().write(|reg| reg.set_value_1(12345)).unwrap();
    let reg = device.foo().read().unwrap();

    assert!(!reg.value_0());
    assert_eq!(reg.value_1(), 12345);
    assert_eq!(reg.value_2(), 0i8);

    device
        .foo()
        .modify(|reg| {
            reg.set_value_0(true);
            reg.set_value_2(-1);
        })
        .unwrap();

    let reg = device.foo().read().unwrap();

    assert!(reg.value_0());
    assert_eq!(reg.value_1(), 12345);
    assert_eq!(reg.value_2(), -1);

    assert_eq!(
        &device.interface.device_memory[0..3],
        &[(0x39 << 1) + 1, 0x30 << 1, 0xFF]
    );
}

#[test]
#[should_panic]
fn test_repeated_too_large_index() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device.foo_repeated(4);
}

#[test]
fn test_repeated_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());
    device
        .foo_repeated(2)
        .modify(|reg| {
            reg.set_value_0(true);
            reg.set_value_1(12345);
            reg.set_value_2(-1);
        })
        .unwrap();

    assert_eq!(
        &device.interface.device_memory[9..12],
        &[(0x39 << 1) + 1, 0x30 << 1, 0xFF]
    );
}
