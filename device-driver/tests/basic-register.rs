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

#[test]
fn test_multi_read_modify_write() {
    let mut device = MyTestDevice::new(DeviceInterface::new());

    let mut multi = device
        .multi(|d| d.foo())
        .and(|d| d.foo_repeated(0));
        // .and(|d| d.foo_repeated(1))
        // .and(|d| d.foo_repeated(2));

    multi.write(|(foo, foo_r0/*, foo_r1, foo_r2*/)| {
        foo.set_value_1(0x01);
        foo_r0.set_value_1(0x02);
        // foo_r1.set_value_1(0x03);
        // foo_r2.set_value_1(0x04);
    });

    assert_eq!(
        &device.interface.device_memory[0..12],
        [0x02, 0x00, 0x00, 0x04, 0x00, 0x00, 0x06, 0x00, 0x00, 0x08, 0x00, 0x00]
    );
}

use device_driver::FieldSet;

impl<I> MyTestDevice<I> {
    pub fn multi<AddressType: Copy, Register: FieldSet, Access>(
        &mut self,
        f: impl FnOnce(&mut Self) -> RegisterOperation<'_, I, AddressType, Register, Access>,
    ) -> MultiRegisterOperation<Self, I, AddressType, FieldSetConstructor<Register>> {
        let operation = f(self);
        let start_address = operation.address();
        let register_new_with_reset = operation.register_new_with_reset();

        MultiRegisterOperation {
            block: self,
            start_address,
            constructors: FieldSetConstructor {
                constructor: register_new_with_reset,
            },
            _phantom: PhantomData,
        }
    }
}

pub struct MultiRegisterOperation<'b, Block, I, AddressType: Copy, FSConstructors> {
    block: &'b mut Block,
    start_address: AddressType,
    constructors: FSConstructors,
    _phantom: PhantomData<(I, AddressType)>,
}

impl<'b, Block, I, AddressType: Copy, FSConstructors>
    MultiRegisterOperation<'b, Block, I, AddressType, FSConstructors>
{
    pub fn and<Register: FieldSet, Access>(
        self,
        f: impl FnOnce(&mut Block) -> RegisterOperation<'_, I, AddressType, Register, Access>,
    ) -> MultiRegisterOperation<'b, Block, I, AddressType, (FSConstructors, FieldSetConstructor<Register>)> {
        let operation = f(self.block);
        let register_new_with_reset = operation.register_new_with_reset();

        let Self {
            block,
            start_address,
            constructors: data,
            _phantom,
        } = self;

        MultiRegisterOperation {
            block,
            start_address,
            constructors: (
                data,
                FieldSetConstructor {
                    constructor: register_new_with_reset,
                },
            ),
            _phantom,
        }
    }
}

impl<'b, Block, I, AddressType: Copy, FSConstructors: NestedFSConstructors>
    MultiRegisterOperation<'b, Block, I, AddressType, FSConstructors>
{
    pub fn write(&mut self, _f: impl FnOnce(&mut FSConstructors::Flattened)) {
        todo!()
    }
}

pub struct FieldSetConstructor<FS: FieldSet> {
    constructor: fn() -> FS,
}

pub trait NestedFSConstructors {
    type Flattened: MultiFieldSet;

    fn new_flattened(self) -> Self::Flattened;
    fn new_with_zero_flattened(self) -> Self::Flattened;
}

impl<R0: FieldSet> NestedFSConstructors for FieldSetConstructor<R0> {
    type Flattened = R0;

    fn new_flattened(self) -> Self::Flattened {
        (self.constructor)()
    }

    fn new_with_zero_flattened(self) -> Self::Flattened {
        R0::new_with_zero()
    }
}

impl<R0: FieldSet, R1: FieldSet> NestedFSConstructors for (FieldSetConstructor<R0>, FieldSetConstructor<R1>) {
    type Flattened = (R0, R1);

    fn new_flattened(self) -> Self::Flattened {
        ((self.0.constructor)(), (self.1.constructor)())
    }

    fn new_with_zero_flattened(self) -> Self::Flattened {
        (R0::new_with_zero(), R1::new_with_zero())
    }
}

pub trait MultiFieldSet {
    type BUFFER: AsMut<[u8]> + AsRef<[u8]>;
    fn to_buffer(self) -> Self::BUFFER;
    fn from_buffer(buffer: Self::BUFFER) -> Self;
}

impl<R0: FieldSet> MultiFieldSet for R0 {
    type BUFFER = R0::BUFFER;

    fn to_buffer(self) -> Self::BUFFER {
        self.into()
    }

    fn from_buffer(buffer: Self::BUFFER) -> Self {
        buffer.into()
    }
}

// error[E0119]: conflicting implementations of trait `MultiFieldSet` for type `(_, _)`
// ^ is fixed by moving this into the d-d crate itself
impl<R0: FieldSet, R1: FieldSet> MultiFieldSet for (R0, R1) {
    type BUFFER = [u8; (R0::SIZE_BITS + R1::SIZE_BITS).div_ceil(8) as usize];

    fn to_buffer(self) -> Self::BUFFER {
        todo!()
    }

    fn from_buffer(buffer: Self::BUFFER) -> Self {
        todo!()
    }
}
