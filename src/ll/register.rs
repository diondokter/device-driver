use crate::hl::Device;

/// This device contains registers
pub trait RegisterDevice<'a, I: RegisterInterface, P>: Device<I, P> {
    type RegisterSet: RegisterSet<'a, I, P>;

    /// Get access to the registers. This access borrows the interface.
    /// *NOTE* Using this can conflict with high-level functionality. Make sure not to break any assumptions that the crate makes.
    fn registers(&'a mut self) -> Self::RegisterSet;
}

/// This is the low-level access to the registers
pub trait RegisterSet<'a, I: RegisterInterface, P>: Sized {
    fn new(interface: &'a mut I, pins: &'a mut P) -> Self
    where
        Self: Sized + 'a;
}

pub enum RegisterError<IE> {
    InvalidValue,
    InterfaceError(IE),
}

impl<IE> From<IE> for RegisterError<IE> {
    fn from(value: IE) -> Self {
        RegisterError::InterfaceError(value)
    }
}

pub trait RegisterInterface {
    type Word;
    type Address;
    type InterfaceError;

    fn read_register(
        &mut self,
        address: Self::Address,
    ) -> Result<Self::Word, RegisterError<Self::InterfaceError>>;
    fn write_register(
        &mut self,
        address: Self::Address,
        value: Self::Word,
    ) -> Result<(), RegisterError<Self::InterfaceError>>;
}

#[macro_export]
macro_rules! implement_registers {
    (
        $device_name:ident {
            $(
                $register_name:ident
            ),*
        }
    ) => {
        use device_driver::ll::register::{RegisterDevice, RegisterInterface, RegisterSet};
        pub mod registers {
            use super::*;
            use device_driver::hl::Device;

            impl<'a, I, P> RegisterDevice<'a, I, P> for $device_name<I, P>
            where
                I: 'a + RegisterInterface + InterfaceBounds,
                P: 'a + PinsBounds,
            {
                type RegisterSet = Registers<'a, I, P>;

                fn registers(&'a mut self) -> Self::RegisterSet {
                    RegisterSet::new(&mut self.interface, &mut self.pins)
                }
            }

            pub struct Registers<'a, I, P>
            where
                I: 'a + RegisterInterface + InterfaceBounds,
                P: 'a + PinsBounds,
            {
                interface: &'a mut I,
                pins: &'a mut P,
            }

            pub struct RegAccessor<'a, I, P, R, W>
            where
                I: 'a + RegisterInterface + InterfaceBounds,
                P: 'a + PinsBounds,
            {
                interface: &'a mut I,
                pins: &'a mut P,
                phantom: core::marker::PhantomData<(R, W)>,
            }

            impl<'a, I, P, R, W> RegAccessor<'a, I, P, R, W>
            where
                I: 'a + RegisterInterface + InterfaceBounds,
                P: 'a + PinsBounds,
            {
                fn new(interface: &'a mut I, pins: &'a mut P) -> Self {
                    Self {
                        interface,
                        pins,
                        phantom: Default::default(),
                    }
                }
            }

            impl<'a, I, P> RegisterSet<'a, I, P> for Registers<'a, I, P>
            where
                I: 'a + RegisterInterface + InterfaceBounds,
                P: 'a + PinsBounds,
            {
                fn new(interface: &'a mut I, pins: &'a mut P) -> Self
                where
                    Self: Sized + 'a,
                {
                    Self { interface, pins }
                }
            }

            impl<'a, I, P> Registers<'a, I, P>
            where
                I: 'a + RegisterInterface + InterfaceBounds,
                P: 'a + PinsBounds,
            {
                $(
                    pub fn $register_name(&'a mut self) -> RegAccessor<'a, I, P, $register_name::W, $register_name::R> {
                        RegAccessor::new(&mut self.interface, &mut self.pins)
                    }
                )*
            }

            $(
                mod $register_name {
                    use super::*;

                    pub struct W;
                    pub struct R;
                }
            )*


        }
    };
}
