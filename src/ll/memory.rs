use crate::hl::Device;

/// This device has a memory space (like eeprom, or a radio rx/tx buffer)
pub trait MemoryDevice<'a, Interface> : Device<Interface> {
    type MemorySpace: MemorySpace<'a, Interface, Self::Error>;
    
    /// Get access to the memory space. This access borrows the interface.
    /// *NOTE* Using this can conflict with high-level functionality. Make sure not to break any assumptions that the crate makes.
    fn memory(&'a mut self) -> Self::MemorySpace;
}

/// This is the memory space
pub trait MemorySpace<'a, Interface, Error> : Sized {
    fn new(interface: &'a mut Interface) -> Self
        where Self: Sized + 'a;
}
