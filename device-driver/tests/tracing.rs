use std::sync::{
    Mutex,
    atomic::{AtomicU8, Ordering},
};

use critical_section::RestoreState;
use device_driver::{FieldsetMetadata, RegisterInterface, RegisterInterfaceBase, TraceInterface};
use device_driver_trace::{
    ObjectIdentifier,
    decoder::{Event, Operation},
};

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

static TEST_LOCK: Mutex<()> = Mutex::new(());
static mut WRITTEN_BYTES: Vec<u8> = Vec::new();
static mut TAKEN: bool = false;
static mut RESTORE_STATE: RestoreState = RestoreState::invalid();

static INTERFACE: TraceInterface = TraceInterface {
    acquire: || {
        unsafe { RESTORE_STATE = critical_section::acquire() };
        if unsafe { TAKEN } {
            panic!("trace interface taken reentrantly");
        }
    },
    write: |buf| unsafe {
        #[allow(static_mut_refs)]
        WRITTEN_BYTES.extend(buf);
    },
    timestamp: || Some(42),
    release: || unsafe {
        TAKEN = false;
        critical_section::release(RESTORE_STATE);
    },
    id: || {
        static ID: AtomicU8 = AtomicU8::new(0);
        ID.fetch_add(1, Ordering::Relaxed)
    },
};

#[test]
fn test_single_write() {
    let _tl = TEST_LOCK.lock().unwrap();
    (INTERFACE.acquire)();
    #[allow(static_mut_refs)]
    unsafe {
        WRITTEN_BYTES.clear()
    };
    unsafe {
        (INTERFACE.release)();
    }

    device_driver::set_global_trace_interface(Some(&INTERFACE));

    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .single_byte()
        .write(|reg| reg.set_value(0xDD))
        .unwrap();

    (INTERFACE.acquire)();
    #[allow(static_mut_refs)]
    let written_bytes = unsafe { WRITTEN_BYTES.clone() };
    unsafe {
        (INTERFACE.release)();
    }

    let mut decoder = device_driver_trace::decoder::StreamingDecoder::new();
    let events = decoder.push_bytes(&written_bytes);

    assert_eq!(
        events,
        vec![Ok(Event {
            instance: 55,
            id: 0,
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
