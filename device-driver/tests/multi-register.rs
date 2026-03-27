use device_driver::{FieldSet, RegisterInterface};

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

device_driver::compile!(
    options: [
        "defmt-feature=defmt"
    ],
    ddsl: "
        device MyTestDevice {
            byte-order: BE,
            register-address-type: u8,

            /// This is the Foo register
            register Foo {
                address: 0,
                fields: fieldset FooFields {
                    size-bits: 20,
                    field value 19:0 -> uint,
                }
            },
            /// This is the Foo register
            register FooRepeated[4*3] {
                address: 3,
                fields: FooFields,
            },
            fieldset BarFields {
                size-bits: 12,
                field value 11:0 -> uint,
            }
        }
    "
);

#[repr(C)]
struct MultiFS<L: FieldSet, R>(L, R);

impl<L: FieldSet, R: SimpleFieldSet> From<(L, R)> for MultiFS<L, R> {
    fn from(value: (L, R)) -> Self {
        let mut multi = MultiFS(value.0, value.1);
        multi.pack_r();
        multi
    }
}

impl<L: FieldSet, R: SimpleFieldSet> MultiFS<L, R> {
    const GAP_BITS: u32 = { core::mem::size_of::<L>() as u32 * 8 - L::SIZE_BITS };

    fn pack_r(&mut self) {
        if Self::GAP_BITS == 0 {
            return;
        }

        println!("pack gap: {}", Self::GAP_BITS);

        let shift_offset = if Self::GAP_BITS % 8 == 0 {
            8
        } else {
            Self::GAP_BITS % 8
        };

        let inner_buf = self.get_inner_buffer_mut();

        let mut overlap_byte_index =
            L::SIZE_BITS.div_ceil(8) as usize - 1 + (shift_offset / 8) as usize;
        let mut source_byte_index = core::mem::size_of::<L>();

        let mut wide_mask = 0x00FF;
        wide_mask <<= shift_offset;

        while source_byte_index < inner_buf.len() {
            println!("target: {overlap_byte_index}, source: {source_byte_index}");

            let mut wide_source = inner_buf[source_byte_index] as u16;

            wide_source <<= shift_offset;

            println!("1. {inner_buf:02X?}");
            inner_buf[overlap_byte_index] &= !(wide_mask >> 8) as u8;
            println!("2. {inner_buf:02X?}");
            inner_buf[overlap_byte_index] |= (wide_source >> 8) as u8;
            println!("3. {inner_buf:02X?}");
            inner_buf[overlap_byte_index + 1] &= !wide_mask as u8;
            println!("4. {inner_buf:02X?}");
            inner_buf[overlap_byte_index + 1] |= wide_source as u8;
            println!("5. {inner_buf:02X?}");

            overlap_byte_index += 1;
            source_byte_index += 1;
        }
    }

    fn unpack_r(&mut self) {
        if Self::GAP_BITS == 0 {
            return;
        }

        println!("unpack gap: {}", Self::GAP_BITS);

        let shift_offset = if Self::GAP_BITS % 8 == 0 {
            0
        } else {
            Self::GAP_BITS % 8
        };

        let inner_buf = self.get_inner_buffer_mut();

        let end_source_byte_index = core::mem::size_of::<L>();
        let end_overlap_byte_index = L::SIZE_BITS.div_ceil(8) as usize - 1;

        for i in (1..=core::mem::size_of::<R>()).rev().map(|i| i - 1) {
            let source_byte_index = end_source_byte_index + i;
            let overlap_byte_index = end_overlap_byte_index + i;

            println!("target: {overlap_byte_index}, source: {source_byte_index}");

            let overlap = ((inner_buf[overlap_byte_index] as u16) << 8)
                | inner_buf[overlap_byte_index + 1] as u16;

            println!(
                "Overlap: {overlap:04X}, shifted: {:02X}",
                (overlap >> shift_offset) as u8
            );
            println!("1. {inner_buf:02X?}");
            inner_buf[source_byte_index] = (overlap >> shift_offset) as u8;
            println!("2. {inner_buf:02X?}");
        }
    }
}

impl<L: FieldSet, R: SimpleFieldSet> Default for MultiFS<L, R> {
    fn default() -> Self {
        let mut val = Self(Default::default(), Default::default());
        val.pack_r();
        val
    }
}

impl<L: FieldSet, R: SimpleFieldSet> FieldSet for MultiFS<L, R> {
    const SIZE_BITS: u32 = L::SIZE_BITS + R::SIZE_BITS;

    fn get_inner_buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (&raw const *self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }

    fn get_inner_buffer_mut(&mut self) -> &mut [u8] {
        unsafe {
            core::slice::from_raw_parts_mut(
                (&raw mut *self).cast::<u8>(),
                core::mem::size_of::<Self>(),
            )
        }
    }
}

trait ToTuple {
    type Tuple;

    fn to_tuple(self) -> Self::Tuple;
}

impl<A: SimpleFieldSet> ToTuple for A {
    type Tuple = A;

    fn to_tuple(self) -> Self::Tuple {
        self
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet> ToTuple for MultiFS<A, B> {
    type Tuple = (A, B);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        (self.0, self.1)
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet, C: SimpleFieldSet> ToTuple
    for MultiFS<MultiFS<A, B>, C>
{
    type Tuple = (A, B, C);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        self.0.unpack_r();
        (self.0.0, self.0.1, self.1)
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet, C: SimpleFieldSet, D: SimpleFieldSet> ToTuple
    for MultiFS<MultiFS<MultiFS<A, B>, C>, D>
{
    type Tuple = (A, B, C, D);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        self.0.unpack_r();
        self.0.0.unpack_r();
        (self.0.0.0, self.0.0.1, self.0.1, self.1)
    }
}

impl<A: SimpleFieldSet, B: SimpleFieldSet, C: SimpleFieldSet, D: SimpleFieldSet, E: SimpleFieldSet>
    ToTuple for MultiFS<MultiFS<MultiFS<MultiFS<A, B>, C>, D>, E>
{
    type Tuple = (A, B, C, D, E);

    fn to_tuple(mut self) -> Self::Tuple {
        self.unpack_r();
        self.0.unpack_r();
        self.0.0.unpack_r();
        self.0.0.0.unpack_r();
        (self.0.0.0.0, self.0.0.0.1, self.0.0.1, self.0.1, self.1)
    }
}

trait SimpleFieldSet: FieldSet {}

impl SimpleFieldSet for BarFields {}
impl SimpleFieldSet for FooFields {}

#[test]
fn simple_layout_ok() {
    let mut foo1_in = FooFields::new();
    foo1_in.set_value(0x12345);
    let mut foo2_in = FooFields::new();
    foo2_in.set_value(0x12345);
    let mut bar_in = BarFields::new();
    bar_in.set_value(0x123);

    let mfs = MultiFS::from((foo1_in, foo2_in));
    let mfs = MultiFS::from((mfs, bar_in));

    assert_eq!(size_bits_of_val(&mfs), 20 + 20 + 12);
    assert_eq!(core::mem::size_of_val(&mfs), 3 + 3 + 2);

    assert_eq!(
        mfs.get_inner_buffer(),
        &[0x11, 0x11, 0x12, 0x22, 0x22, 0x66, 0x77, 0x77]
    );

    let (foo1, foo2, bar) = mfs.to_tuple();

    assert_eq!(foo1, foo1_in);
    assert_eq!(foo2, foo2_in);
    assert_eq!(bar, bar_in);
}

fn size_bits_of_val<T: FieldSet>(_: &T) -> u32 {
    T::SIZE_BITS
}
