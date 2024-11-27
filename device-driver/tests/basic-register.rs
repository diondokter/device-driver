use std::marker::PhantomData;

use device_driver::{RegisterInterface, RegisterOperation};

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
    device_name: MyTestDevice,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type DefaultByteOrder = LE;
        }
        /// This is the Foo register
        register Foo {
            const ADDRESS = 0;
            const SIZE_BITS = 24;

            /// This is a bool!
            value0: bool = 0,
            value1: uint = 1..16,
            value2: int = 16..24,
        },
        /// This is the Foo register
        register FooRepeated {
            const ADDRESS = 3;
            const SIZE_BITS = 24;
            const REPEAT = {
                count: 4,
                stride: 3,
            };

            /// This is a bool!
            value0: bool = 0,
            value1: uint = 1..16,
            value2: int = 16..24,
        }
    }
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

// ------------------------------------------------------------------------------------------

impl MultiRegisterInterface for DeviceInterface {
    type Error = ();
    type AddressType = u8;

    fn write_registers(
        &mut self,
        mut address: Self::AddressType,
        _size_bits: &[u32],
        data: &[&[u8]],
    ) -> Result<(), Self::Error> {
        for data in data {
            self.device_memory[(address as usize)..][..data.len()].copy_from_slice(data);
            address += data.len() as u8;
        }

        Ok(())
    }

    fn read_registers(
        &mut self,
        mut address: Self::AddressType,
        _size_bits: &[u32],
        data: &mut [&mut [u8]],
    ) -> Result<(), Self::Error> {
        for data in data {
            data.copy_from_slice(&self.device_memory[address as usize..][..data.len()]);
            address += data.len() as u8;
        }

        Ok(())
    }
}

#[test]
fn test_multi_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    device.foo()
        .and(|d| d.bar())
        .and(|d| d.cow())
        .write(|(a, b, c)| {
            a.set_x(false);
            b.set_y(123);
            c.set_x(Cow::SomeValue);
        });

    device
        .multi(|d| (d.foo(), d.foo_repeated(0)))
        .write(|(foo, foo_r0)| {
            foo.set_value_1(0x01);
            foo_r0.set_value_1(0x02);
            // foo_r1.set_value_1(0x03);
            // foo_r2.set_value_1(0x04);
        })
        .unwrap();

    assert_eq!(
        &device.interface.device_memory[0..12],
        [0x02, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]
        // [0x02, 0x00, 0x00, 0x04, 0x00, 0x00, 0x06, 0x00, 0x00, 0x08, 0x00, 0x00]
    );

    let (foo, foo_r0) = device
        .multi(|d| (d.foo(), d.foo_repeated(0)))
        .read()
        .unwrap();

    assert_eq!(foo.value_1(), 0x01);
    assert_eq!(foo_r0.value_1(), 0x02);
}

// Generated by macro
impl<I> MyTestDevice<I> {
    pub fn multi<Addr: Copy, C: Chain<Addr>>(
        &mut self,
        f: impl FnOnce(&mut MyTestDevice<()>) -> C,
    ) -> MultiRegisterOperation<'_, I, Addr, C> {
        let chain = f(&mut MyTestDevice::new(()));

        MultiRegisterOperation {
            interface: &mut self.interface,
            chain,
            _phantom: PhantomData,
        }
    }
}

// Put in d-d
pub struct MultiRegisterOperation<'i, I, Addr: Copy, Chain> {
    interface: &'i mut I,
    chain: Chain,
    _phantom: PhantomData<Addr>,
}

// Put in d-d
impl<'i, I, Addr: Copy, C: Chain<Addr>> MultiRegisterOperation<'i, I, Addr, C> {
    pub fn write<R>(&mut self, f: impl FnOnce(&mut C::Fieldsets) -> R) -> Result<R, I::Error>
    where
        I: MultiRegisterInterface<AddressType = Addr>,
        Addr: Copy,
    {
        let mut registers = self.chain.create();
        let returned = f(&mut registers);

        let buffers = C::to_buffers(registers);
        let slices = C::as_slices(&buffers);

        self.interface
            .write_registers(self.chain.start_address(), &[], slices.as_ref())?;

        Ok(returned)
    }

    pub fn write_with_zero<R>(
        &mut self,
        f: impl FnOnce(&mut C::Fieldsets) -> R,
    ) -> Result<R, I::Error>
    where
        I: MultiRegisterInterface<AddressType = Addr>,
        Addr: Copy,
    {
        let mut registers = C::create_zero();
        let returned = f(&mut registers);

        let buffers = C::to_buffers(registers);
        let slices = C::as_slices(&buffers);

        self.interface
            .write_registers(self.chain.start_address(), &[], slices.as_ref())?;

        Ok(returned)
    }

    pub fn read(&mut self) -> Result<C::Fieldsets, I::Error>
    where
        I: MultiRegisterInterface<AddressType = Addr>,
        Addr: Copy,
    {
        let mut buffers = C::to_buffers(C::create_zero());
        let mut slices = C::as_slices_mut(&mut buffers);

        self.interface
            .read_registers(self.chain.start_address(), &[], slices.as_mut())?;
        drop(slices);

        Ok(C::to_fieldsets(buffers))
    }
}

// Put in d-d doc hidden
pub trait Chain<Addr: Copy> {
    type Fieldsets;
    type Buffers;
    type SlicesMut<'a>: AsMut<[&'a mut [u8]]>;
    type Slices<'a>: AsRef<[&'a [u8]]>;

    fn start_address(&self) -> Addr;
    fn addresses(&self) -> impl Iterator<Item = Addr>;

    fn create(&self) -> Self::Fieldsets;
    fn create_zero() -> Self::Fieldsets;

    fn to_buffers(fs: Self::Fieldsets) -> Self::Buffers;
    fn to_fieldsets(buffers: Self::Buffers) -> Self::Fieldsets;

    fn as_slices_mut(buffers: &mut Self::Buffers) -> Self::SlicesMut<'_>;
    fn as_slices(buffers: &Self::Buffers) -> Self::Slices<'_>;
}

impl<AddressType, R0, R1, Access> Chain<AddressType>
    for (
        RegisterOperation<'static, (), AddressType, R0, Access>,
        RegisterOperation<'static, (), AddressType, R1, Access>,
    )
where
    AddressType: Copy,
    R0: device_driver::FieldSet,
    R1: device_driver::FieldSet,
{
    type Fieldsets = (R0, R1);
    type Buffers = (R0::BUFFER, R1::BUFFER);
    type SlicesMut<'a> = [&'a mut [u8]; 2];
    type Slices<'a> = [&'a [u8]; 2];

    fn start_address(&self) -> AddressType {
        self.0.address()
    }

    fn addresses(&self) -> impl Iterator<Item = AddressType> {
        [self.0.address(), self.1.address()].into_iter()
    }

    fn create(&self) -> Self::Fieldsets {
        (self.0.new_fs(), self.1.new_fs())
    }

    fn create_zero() -> Self::Fieldsets {
        (R0::new_with_zero(), R1::new_with_zero())
    }

    fn to_buffers(fs: Self::Fieldsets) -> Self::Buffers {
        (fs.0.into(), fs.1.into())
    }

    fn to_fieldsets(buffers: Self::Buffers) -> Self::Fieldsets {
        (buffers.0.into(), buffers.1.into())
    }

    fn as_slices_mut(buffers: &mut Self::Buffers) -> Self::SlicesMut<'_> {
        [buffers.0.as_mut(), buffers.1.as_mut()]
    }
    fn as_slices(buffers: &Self::Buffers) -> Self::Slices<'_> {
        [buffers.0.as_ref(), buffers.1.as_ref()]
    }
}

// Put in d-d crate

pub trait MultiRegisterInterface {
    /// The error type
    type Error;
    /// The address type used by this interface. Should likely be an integer.
    type AddressType: Copy;

    /// Write the given data to the registers located at the given address and continuing
    fn write_registers(
        &mut self,
        address: Self::AddressType,
        size_bits: &[u32],
        data: &[&[u8]],
    ) -> Result<(), Self::Error>;

    /// Read the registers located at the given addres to the given data slice
    fn read_registers(
        &mut self,
        address: Self::AddressType,
        size_bits: &[u32],
        data: &mut [&mut [u8]],
    ) -> Result<(), Self::Error>;
}
