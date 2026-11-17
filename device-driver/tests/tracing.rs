use std::sync::Mutex;

use device_driver::{Block, FieldsetMetadata, RegisterInterface, RegisterInterfaceBase};
use device_driver_trace::{
    ObjectIdentifier, TestTraceInterface, crc32,
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

// `cargo test` shares global state for tests, so we need to 'serialize' the tests using this lock
static TEST_LOCK: Mutex<()> = Mutex::new(());
static INTERFACE: TestTraceInterface = TestTraceInterface::new();

#[test]
fn test_single_modify() {
    let _tl = TEST_LOCK.lock().unwrap();
    let _ = device_driver::set_global_trace_interface(&INTERFACE);
    INTERFACE.reset();
    INTERFACE.set_timestamp(Some(1_001_000));

    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .my_register()
        .modify(|reg| {
            reg.set_ccc(Ccc::A);
            reg.set_aaa(true);
            reg.set_ddd(540);
        })
        .unwrap();

    let written_bytes = INTERFACE.take_bytes();

    let mut decoder = device_driver_trace::decoder::StreamingDecoder::new();
    let events = decoder.push_bytes(&written_bytes);

    assert_eq!(
        events,
        vec![Ok(Event {
            source_crc: device_driver_trace::crc32(include_bytes!("./tracing.ddsl")),
            id: 0,
            timestamp: Some(1_001_000),
            operations: vec![Operation {
                objects: vec![ObjectIdentifier::new(0x03000000, None)],
                address: vec![0],
                read_data: vec![0, 0, 0, 0],
                write_data: vec![0xD1, 0x80, 0x43, 0x00],
            }]
        })]
    );

    let trace = render_trace(&events);
    println!("{trace}");
}

#[test]
fn test_bulk_modify() {
    let _tl = TEST_LOCK.lock().unwrap();
    let _ = device_driver::set_global_trace_interface(&INTERFACE);
    INTERFACE.reset();
    INTERFACE.set_timestamp(None);

    let mut device = MyTestDevice::new(DeviceInterface::new());

    device
        .bulk_modify()
        .with(|d| d.my_register().plan())
        .with(|d| d.second().plan_array_at::<4>(0))
        .with(|d| d.third().plan_at(ThirdIndex::A))
        .execute(|(x, y, z)| {
            x.set_aaa(true);
            x.set_ddd(0xFFF);
            y[0].set_value(42);
            z.set_value(255);
        })
        .unwrap();

    let written_bytes = INTERFACE.take_bytes();

    let mut decoder = device_driver_trace::decoder::StreamingDecoder::new();
    let events = decoder.push_bytes(&written_bytes);

    // assert_eq!(
    //     events,
    //     vec![Ok(Event {
    //         source_crc: device_driver_trace::crc32(include_bytes!("./tracing.ddsl")),
    //         id: 0,
    //         timestamp: Some(1_001_000),
    //         operations: vec![Operation {
    //             objects: vec![ObjectIdentifier::new(0x03000000, None)],
    //             address: vec![0],
    //             read_data: vec![0, 0, 0, 0],
    //             write_data: vec![0xD1, 0x80, 0x43, 0x00],
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

    let manifests = [(crc32(DDSL.as_bytes()), manifest)].into_iter().collect();

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
