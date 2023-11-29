// Input

#[implement_registers]
impl<FOO> MyRegisterDevice<FOO> {
    register Id {
        type RWCapability = ReadOnly;
        const ADDRESS: usize = 12;
        const SIZE_BYTES: usize = 3;

        manufacturer: u16 as Manufacturer = 0..16,
        version: u8 = 16..20,
        edition: u8 as enum Edition {
            One = 1,
            Two,
            Five = 5,
        } = 20..24,
    },
    // ...
}

// Output

impl<FOO> MyRegisterDevice<FOO> {
    pub fn id(
        &mut self,
    ) -> RegisterOperation<'_, Self, Id, { Id::SIZE_BYTES }> {
        RegisterOperation::new(self)
    }
}

struct Id {
    bits: BitArray<[u8; Self::SIZE_BYTES]>,
}

impl Register<{ Self::SIZE_BYTES }> for Id {
    const ZERO: Self = Self {
        bits: BitArray::ZERO,
    };

    type AddressType = usize;
    const ADDRESS: Self::AddressType = 3;

    type RWCapability = ReadOnly;

    fn bits(&mut self) -> &mut BitArray<[u8; Self::SIZE_BYTES]> {
        &mut self.bits
    }
}

impl Id {
    pub const SIZE_BYTES: usize = 3;

    pub fn manufacturer(&mut self) -> Field<'_, Self, Manufacturer, u16, 0, 16, { Self::SIZE_BYTES }> {
        Field::new(self)
    }

    pub fn version(&mut self) -> Field<'_, Self, u8, u8, 16, 20, { Self::SIZE_BYTES }> {
        Field::new(self)
    }

    pub fn edition(&mut self) -> Field<'_, Self, Edition, u8, 20, 24, { Self::SIZE_BYTES }> {
        Field::new(self)
    }
}

#[derive(num_enum::TryFromPrimitive, num_enum::IntoPrimitive, Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
pub enum Edition {
    One = 1,
    Two,
    Five = 5,
}
