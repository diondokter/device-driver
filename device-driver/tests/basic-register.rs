use std::marker::PhantomData;

use device_driver::{FsSet, RegisterInterface};

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
        // assert_eq!(size_bits, 24);
        self.device_memory[address as usize..][..data.len()].copy_from_slice(data);

        Ok(())
    }

    fn read_register(
        &mut self,
        address: Self::AddressType,
        size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        // assert_eq!(size_bits, 24);
        data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
        Ok(())
    }
}

device_driver::create_device!(
    kdl: "
        device MyTestDevice {
            byte-order LE
            register-address-type u8
            /// This is the Foo register
            register Foo {
                address 0
                reset-value 0xFFFFFF
                fields size-bits=24 {
                    /// This is a bool!
                    (bool)value0 @0
                    (uint)value1 @15:1
                    (int)value2 @23:17
                }
            }
            /// This is the Foo register
            register FooRepeated {
                address 3
                repeat count=4 stride=3
                fields size-bits=24 {
                    /// This is a bool!
                    (bool)value0 @0
                    (uint)value1 @15:1
                    (int)value2 @23:16
                }
            }
        }
    "
);

#[test]
fn multi_test() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .multi_write()
        .with(|d| {
            d.foo_repeated(0)
                .plan_write_with_zero(|reg| reg.set_value_1(42))
        })
        .with(|d| d.foo().plan_write(|_| {}))
        .execute()
        .unwrap();

    let multi = device
        .multi_read()
        // TODO: Allow reading multiple. This would return a [FooRepeated;N] that can also impl FieldSet.
        // Maybe N is a const generic on foo_repeated with default 1?
        // We'll also need a check whether it's allowed by the device rules. That's probably an assert.
        .with(|d| d.foo_repeated(0).plan_read())
        .with(|d| d.foo().plan_read())
        .execute()
        .unwrap();

    println!("{multi:?}");

    device
        .multi_modify()
        .with(|d| d.foo_repeated(0).plan_modify())
        .with(|d| d.foo().plan_modify())
        .execute(|(foo_repeat_0, foo)| {
            foo_repeat_0.set_value_2(-5);
            foo.set_value_0(false);
        })
        .unwrap();

    let multi = device
        .multi_read()
        .with(|d| d.foo_repeated(0).plan_read())
        .with(|d| d.foo().plan_read())
        .execute()
        .unwrap();

    println!("{multi:?}");
}

pub struct MultiRegisterOperation<'d, D, AddressType, FieldSets: FsSet, Access> {
    device: &'d mut D,
    start_address: Option<AddressType>,
    field_sets: FieldSets,
    bit_sum: u32,
    _phantom: PhantomData<Access>,
}

impl<'d, D, AddressType, FieldSets: FsSet>
    MultiRegisterOperation<'d, D, AddressType, FieldSets, device_driver::RO>
where
    D: Device,
    AddressType: Copy,
{
    #[inline]
    pub fn with<FS: device_driver::FieldSet, LocalAccess: device_driver::ReadCapability>(
        mut self,
        f: impl FnOnce(&mut D) -> device_driver::Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'d, D, AddressType, FieldSets::Next<FS>, device_driver::RO>
    where
        FieldSets::Next<FS>: FsSet,
    {
        let plan = f(self.device);
        if self.start_address.is_none() {
            self.start_address = Some(plan.address)
        }
        assert!(FS::SIZE_BITS.is_multiple_of(8));

        // TODO: Check if legal

        MultiRegisterOperation {
            device: self.device,
            start_address: self.start_address,
            field_sets: self.field_sets.push(plan.value),
            bit_sum: self.bit_sum + FS::SIZE_BITS,
            _phantom: PhantomData,
        }
    }
}

impl<'d, D, AddressType, FieldSets: FsSet>
    MultiRegisterOperation<'d, D, AddressType, FieldSets, device_driver::WO>
where
    D: Device,
    AddressType: Copy,
{
    #[inline]
    pub fn with<FS: device_driver::FieldSet, LocalAccess: device_driver::WriteCapability>(
        mut self,
        f: impl FnOnce(&mut D) -> device_driver::Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'d, D, AddressType, FieldSets::Next<FS>, device_driver::WO>
    where
        FieldSets::Next<FS>: FsSet,
    {
        let plan = f(self.device);
        if self.start_address.is_none() {
            self.start_address = Some(plan.address)
        }
        assert!(FS::SIZE_BITS.is_multiple_of(8));

        // TODO: Check if legal

        MultiRegisterOperation {
            device: self.device,
            start_address: self.start_address,
            field_sets: self.field_sets.push(plan.value),
            bit_sum: self.bit_sum + FS::SIZE_BITS,
            _phantom: PhantomData,
        }
    }
}

impl<'d, D, AddressType, FieldSets: FsSet>
    MultiRegisterOperation<'d, D, AddressType, FieldSets, device_driver::RW>
where
    D: Device,
    AddressType: Copy,
{
    #[inline]
    pub fn with<
        FS: device_driver::FieldSet,
        LocalAccess: device_driver::WriteCapability + device_driver::ReadCapability,
    >(
        mut self,
        f: impl FnOnce(&mut D) -> device_driver::Plan<AddressType, FS, LocalAccess>,
    ) -> MultiRegisterOperation<'d, D, AddressType, FieldSets::Next<FS>, device_driver::RW>
    where
        FieldSets::Next<FS>: FsSet,
    {
        let plan = f(self.device);
        if self.start_address.is_none() {
            self.start_address = Some(plan.address)
        }
        assert!(FS::SIZE_BITS.is_multiple_of(8));

        // TODO: Check if legal

        MultiRegisterOperation {
            device: self.device,
            start_address: self.start_address,
            field_sets: self.field_sets.push(plan.value),
            bit_sum: self.bit_sum + FS::SIZE_BITS,
            _phantom: PhantomData,
        }
    }
}

impl<'d, D, FieldSets: FsSet>
    MultiRegisterOperation<
        'd,
        D,
        <D::Interface as device_driver::RegisterInterface>::AddressType,
        FieldSets,
        device_driver::RO,
    >
where
    D: Device,
    D::Interface: device_driver::RegisterInterface,
{
    #[inline]
    pub fn execute(
        mut self,
    ) -> Result<FieldSets::Value, <D::Interface as device_driver::RegisterInterface>::Error> {
        let data = self.field_sets.as_slice_mut();
        self.device
            .interface()
            .read_register(self.start_address.unwrap(), self.bit_sum, data)?;
        Ok(self.field_sets.to_value())
    }
}

impl<'d, D, FieldSets: FsSet>
    MultiRegisterOperation<
        'd,
        D,
        <D::Interface as device_driver::RegisterInterface>::AddressType,
        FieldSets,
        device_driver::WO,
    >
where
    D: Device,
    D::Interface: device_driver::RegisterInterface,
{
    #[inline]
    pub fn execute(
        mut self,
    ) -> Result<FieldSets::Value, <D::Interface as device_driver::RegisterInterface>::Error> {
        let data = self.field_sets.as_slice_mut();
        self.device
            .interface()
            .write_register(self.start_address.unwrap(), self.bit_sum, data)?;
        Ok(self.field_sets.to_value())
    }
}

impl<'d, D, FieldSets: FsSet>
    MultiRegisterOperation<
        'd,
        D,
        <D::Interface as device_driver::RegisterInterface>::AddressType,
        FieldSets,
        device_driver::RW,
    >
where
    D: Device,
    D::Interface: device_driver::RegisterInterface,
{
    #[inline]
    pub fn execute<R>(
        mut self,
        f: impl FnOnce(FieldSets::ValueMut<'_>) -> R,
    ) -> Result<R, <D::Interface as device_driver::RegisterInterface>::Error> {
        self.device.interface().read_register(
            self.start_address.unwrap(),
            self.bit_sum,
            self.field_sets.as_slice_mut(),
        )?;

        let returned = f(self.field_sets.as_value_mut());

        self.device.interface().write_register(
            self.start_address.unwrap(),
            self.bit_sum,
            self.field_sets.as_slice_mut(),
        )?;

        Ok(returned)
    }
}

pub trait Device {
    type Interface;
    type AddressType;

    fn interface(&mut self) -> &mut Self::Interface;

    fn multi_read(
        &mut self,
    ) -> MultiRegisterOperation<'_, Self, Self::AddressType, (), device_driver::RO>
    where
        Self: Sized,
    {
        MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            bit_sum: 0,
            _phantom: PhantomData,
        }
    }

    fn multi_write(
        &mut self,
    ) -> MultiRegisterOperation<'_, Self, Self::AddressType, (), device_driver::WO>
    where
        Self: Sized,
    {
        MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            bit_sum: 0,
            _phantom: PhantomData,
        }
    }

    fn multi_modify(
        &mut self,
    ) -> MultiRegisterOperation<'_, Self, Self::AddressType, (), device_driver::RW>
    where
        Self: Sized,
    {
        MultiRegisterOperation {
            device: self,
            start_address: None,
            field_sets: (),
            bit_sum: 0,
            _phantom: PhantomData,
        }
    }
}

impl<I> Device for MyTestDevice<I> {
    type Interface = I;
    type AddressType = u8;

    fn interface(&mut self) -> &mut Self::Interface {
        self.interface()
    }
}

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
        &[(0x39 << 1) + 1, 0x30 << 1, 0xFE]
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
