use std::sync::{
    Mutex,
    atomic::{AtomicU8, Ordering},
};

use critical_section::RestoreState;
use device_driver::{FieldsetMetadata, RegisterInterface, RegisterInterfaceBase, TraceInterface};
use device_driver_trace::{
    ObjectIdentifier,
    decoder::{DecodeError, Event, Operation},
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
    manifest: "tests/tracing.ddsl"
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
    timestamp: || Some(1_001_000),
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
        .modify(|reg| {
            reg.set_value(Value::A);
            reg.set_fbool(true);
        })
        .unwrap();

    (INTERFACE.acquire)();
    #[allow(static_mut_refs)]
    let written_bytes = unsafe { WRITTEN_BYTES.clone() };
    unsafe {
        (INTERFACE.release)();
    }

    let mut decoder = device_driver_trace::decoder::StreamingDecoder::new();
    let events = decoder.push_bytes(&written_bytes);

    // assert_eq!(
    //     events,
    //     vec![Ok(Event {
    //         instance: 55,
    //         id: 0,
    //         timestamp: Some(1_001_000),
    //         operations: vec![Operation {
    //             objects: vec![ObjectIdentifier::new(0x03000000, None)],
    //             address: vec![0],
    //             read_data: vec![],
    //             write_data: vec![0xD],
    //         }]
    //     })]
    // );

    let trace = render_trace(&events);
    println!("{trace}");
}

fn render_trace(events: &[Result<Event, DecodeError>]) -> String {
    use std::fmt::Write;

    const DDSL: &'static str = include_str!("./tracing.ddsl");

    let mut diagnostics = Default::default();
    let tokens = device_driver_lexer::lex(DDSL);
    let ast = device_driver_parser::parse(&tokens, &mut diagnostics);
    let (manifest, _) =
        device_driver_mir::lower_ast(&ast, &Default::default(), &mut diagnostics).unwrap();
    assert_eq!(diagnostics.summary(), Default::default());

    let manifests = [(55, &manifest)].into_iter().collect();

    let mut output = String::new();

    for event in events {
        match event {
            Ok(event) => {
                match device_driver_trace::correlator::correlate(&manifests, event.clone()) {
                    Ok(event) => writeln!(&mut output, "{event}").unwrap(),
                    Err(e) => writeln!(&mut output, "Event correlate error: {e}").unwrap(),
                }
            }
            Err(e) => writeln!(&mut output, "Event decode error: {e:?}").unwrap(),
        }
    }

    output
}
