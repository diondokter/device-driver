#![cfg_attr(not(any(test, feature = "std")), no_std)]

use core::mem::MaybeUninit;

mod ansi;
#[cfg(any(test, feature = "std"))]
pub(crate) mod bitdump;
#[cfg(any(test, feature = "std"))]
pub mod correlator;
#[cfg(any(test, feature = "std"))]
pub mod decoder;
pub mod encoder;
mod rzcobs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MessageType {
    #[cfg_attr(not(any(test, feature = "std")), expect(unused))]
    End = 0,
    Timestamp = 1,
    ObjectIdentifier = 2,
    OperationData = 3,
    OperationAddress = 4,
    Finalize = 5,
}

impl MessageType {
    #[cfg_attr(not(any(test, feature = "std")), expect(unused))]
    const MASK: u8 = 0b0000_0111;

    #[cfg_attr(not(any(test, feature = "std")), expect(unused))]
    fn from_byte(byte: u8) -> Option<Self> {
        match byte & Self::MASK {
            0 => Some(MessageType::End),
            1 => Some(MessageType::Timestamp),
            2 => Some(MessageType::ObjectIdentifier),
            3 => Some(MessageType::OperationData),
            4 => Some(MessageType::OperationAddress),
            5 => Some(MessageType::Finalize),
            _ => None,
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectIdentifier {
    object_id: u32,
    index: Option<i32>,
}

impl ObjectIdentifier {
    pub fn new(object_id: u32, index: Option<i32>) -> Self {
        Self { object_id, index }
    }
}

pub fn crc32(data: &str) -> u32 {
    const POLY: u32 = 0x82f6_3b78; // Castagnoli

    let mut crc = 0xFFFF_FFFF;

    for byte in data.bytes() {
        if byte == b'\r' {
            // Skip carriage-return to get the crc's across operating systems
            continue;
        }

        crc ^= u32::from(byte);

        for _ in 0..8 {
            let lowest_bit_set = crc & 1 > 0;
            crc >>= 1;
            if lowest_bit_set {
                crc ^= POLY;
            }
        }
    }

    !crc
}

pub trait Write {
    fn write(&mut self, buf: &[u8]);
}

#[cfg(any(test, feature = "std"))]
impl Write for Vec<u8> {
    fn write(&mut self, buf: &[u8]) {
        self.extend_from_slice(buf);
    }
}

pub trait TraceInterface {
    /// Write data into the sink
    fn with_writer(&self, f: &mut dyn FnMut(&mut dyn Write));
    /// Get the current time in microseconds.
    /// If a timestamp is not desired, return None.
    fn timestamp(&self) -> Option<u64>;
    /// Get a new id for trace events.
    /// Should increment by 1 every time when called and wrap around to 0 at the end.
    fn next_id(&self) -> u8;
}

#[cfg(any(test, feature = "std"))]
#[derive(Debug)]
pub struct TestTraceInterface {
    bytes: std::sync::Mutex<Vec<u8>>,
    timestamp: std::sync::Mutex<Option<u64>>,
    next_id: std::sync::atomic::AtomicU8,
}

#[cfg(any(test, feature = "std"))]
impl TestTraceInterface {
    pub const fn new() -> Self {
        Self {
            bytes: std::sync::Mutex::new(Vec::new()),
            timestamp: std::sync::Mutex::new(None),
            next_id: std::sync::atomic::AtomicU8::new(0),
        }
    }

    pub fn take_bytes(&self) -> Vec<u8> {
        self.bytes.lock().unwrap().split_off(0)
    }

    pub fn set_timestamp(&self, value: Option<u64>) {
        *self.timestamp.lock().unwrap() = value;
    }

    pub fn reset(&self) {
        self.take_bytes();
        self.set_timestamp(None);
        self.next_id.store(0, Ordering::Relaxed);
    }
}

#[cfg(any(test, feature = "std"))]
impl Default for TestTraceInterface {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(any(test, feature = "std"))]
impl TraceInterface for TestTraceInterface {
    fn with_writer(&self, f: &mut dyn FnMut(&mut dyn Write)) {
        let mut bytes = self.bytes.lock().unwrap();
        f(&mut *bytes);
    }

    fn timestamp(&self) -> Option<u64> {
        *self.timestamp.lock().unwrap()
    }

    fn next_id(&self) -> u8 {
        self.next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

// Below is copied and adopted from the `log` crate, licensed under MIT.

#[cfg(target_has_atomic = "ptr")]
use core::sync::atomic::{AtomicUsize, Ordering};

#[cfg(not(target_has_atomic = "ptr"))]
use core::cell::Cell;
#[cfg(not(target_has_atomic = "ptr"))]
use core::sync::atomic::Ordering;

#[cfg(not(target_has_atomic = "ptr"))]
struct AtomicUsize {
    v: Cell<usize>,
}

#[cfg(not(target_has_atomic = "ptr"))]
impl AtomicUsize {
    const fn new(v: usize) -> AtomicUsize {
        AtomicUsize { v: Cell::new(v) }
    }

    fn load(&self, _order: Ordering) -> usize {
        self.v.get()
    }

    fn store(&self, val: usize, _order: Ordering) {
        self.v.set(val)
    }
}

// Any platform without atomics is unlikely to have multiple cores, so
// writing via Cell will not be a race condition.
#[cfg(not(target_has_atomic = "ptr"))]
unsafe impl Sync for AtomicUsize {}

// The TRACE_INTERFACE static holds a pointer to the global logger. It is protected by
// the STATE static which determines whether TRACE_INTERFACE has been initialized yet.
static mut TRACE_INTERFACE: MaybeUninit<&dyn TraceInterface> = core::mem::MaybeUninit::uninit();

static STATE: AtomicUsize = AtomicUsize::new(0);

// There are three different states that we care about: the logger's
// uninitialized, the logger's initializing (set_logger's been called but
// TRACE_INTERFACE hasn't actually been set yet), or the logger's active.
const UNINITIALIZED: usize = 0;
const INITIALIZING: usize = 1;
const INITIALIZED: usize = 2;

/// Sets the global trace interface to a `&'static TraceInterface`.
///
/// This function may only be called once in the lifetime of a program. Any trace
/// events that occur before the call to `set_trace_interface` completes will be ignored.
///
/// # Availability
///
/// This method is available even when the `std` feature is disabled. However,
/// it is currently unavailable on `thumbv6` targets, which lack support for
/// some atomic operations which are used by this function. Even on those
/// targets, [`set_trace_interface_racy`] will be available.
///
/// # Errors
///
/// An error is returned if a trace interface has already been set.
#[cfg(target_has_atomic = "ptr")]
pub fn set_trace_interface(
    logger: &'static dyn TraceInterface,
) -> Result<(), SetTraceInterfaceError> {
    match STATE.compare_exchange(
        UNINITIALIZED,
        INITIALIZING,
        Ordering::Acquire,
        Ordering::Relaxed,
    ) {
        Ok(UNINITIALIZED) => {
            unsafe {
                TRACE_INTERFACE = MaybeUninit::new(logger);
            }
            STATE.store(INITIALIZED, Ordering::Release);
            Ok(())
        }
        Err(INITIALIZING) => {
            while STATE.load(Ordering::Relaxed) == INITIALIZING {
                std::hint::spin_loop();
            }
            Err(SetTraceInterfaceError(()))
        }
        _ => Err(SetTraceInterfaceError(())),
    }
}

/// A thread-unsafe version of [`set_trace_interface`].
///
/// This function is available on all platforms, even those that do not have
/// support for atomics that is needed by [`set_trace_interface`].
///
/// In almost all cases, [`set_trace_interface`] should be preferred.
///
/// # Safety
///
/// This function is only safe to call when it cannot race with any other
/// calls to `set_trace_interface` or `set_trace_interface_racy`.
///
/// This can be upheld by (for example) making sure that **there are no other
/// threads**, and that **interrupts are disabled**.
pub unsafe fn set_trace_interface_racy(
    logger: &'static dyn TraceInterface,
) -> Result<(), SetTraceInterfaceError> {
    match STATE.load(Ordering::Acquire) {
        UNINITIALIZED => {
            unsafe {
                TRACE_INTERFACE = MaybeUninit::new(logger);
            }
            STATE.store(INITIALIZED, Ordering::Release);
            Ok(())
        }
        INITIALIZING => {
            // This is just plain UB, since we were racing another initialization function
            unreachable!("set_logger_racy must not be used with other initialization functions")
        }
        _ => Err(SetTraceInterfaceError(())),
    }
}

/// The type returned by [`set_trace_interface`] if [`set_trace_interface`] has already been called.
#[allow(missing_copy_implementations)]
#[derive(Debug)]
pub struct SetTraceInterfaceError(());

impl core::fmt::Display for SetTraceInterfaceError {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_str("attempted to set a trace interface after the system was already initialized")
    }
}
impl core::error::Error for SetTraceInterfaceError {}

/// Returns a reference to the trace interface.
///
/// If a trace interface has not been set, a None is returned.
pub fn trace_interface() -> Option<&'static dyn TraceInterface> {
    // Acquire memory ordering guarantees that current thread would see any
    // memory writes that happened before store of the value
    // into `STATE` with memory ordering `Release` or stronger.
    //
    // Since the value `INITIALIZED` is written only after `TRACE_INTERFACE` was
    // initialized, observing it after `Acquire` load here makes both
    // write to the `TRACE_INTERFACE` static and initialization of the logger
    // internal state synchronized with current thread.
    if STATE.load(Ordering::Acquire) != INITIALIZED {
        None
    } else {
        Some(unsafe { TRACE_INTERFACE.assume_init() })
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        decoder::{Event as DecodeEvent, Operation, StreamingDecoder},
        encoder::Event as EncodeEvent,
    };

    use super::*;

    #[test]
    fn roundtrip_single_events() {
        static INTERFACE: TestTraceInterface = TestTraceInterface::new();
        INTERFACE.set_timestamp(Some(5));
        const SOURCE_CRC: u32 = 54321;

        let none_event = EncodeEvent::new(&INTERFACE, SOURCE_CRC);
        let none_event_id = none_event.id;
        none_event.send_final_frame(&INTERFACE, |f| {
            f.send_timestamp(INTERFACE.timestamp());
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
        });

        let modify_event = EncodeEvent::new(&INTERFACE, SOURCE_CRC);
        let modify_event_id = modify_event.id;
        let modify_event = modify_event.send_frame(&INTERFACE, |f| {
            f.send_timestamp(INTERFACE.timestamp());
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
            f.send_address(&[1]);
            f.send_bytes(false, &[1, 2, 3]);
        });
        modify_event.send_final_frame(&INTERFACE, |f| {
            f.send_bytes(true, &[1, 2, 5]);
        });

        let read_event = EncodeEvent::new(&INTERFACE, SOURCE_CRC);
        let read_event_id = read_event.id;
        read_event.send_final_frame(&INTERFACE, |f| {
            f.send_timestamp(INTERFACE.timestamp());
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
            f.send_address(&[12, 34, 56]);
            f.send_bytes(false, &[1, 2, 3]);
        });

        let written_bytes = INTERFACE.take_bytes();
        println!("Cobs encoded written bytes: {:X?}", &written_bytes);

        let mut decoder = StreamingDecoder::new();
        let decoded_events = decoder
            .push_bytes(&written_bytes)
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>();

        let decoded_none_event = decoded_events
            .iter()
            .find(|e| e.id == none_event_id)
            .unwrap();
        let decoded_modify_event = decoded_events
            .iter()
            .find(|e| e.id == modify_event_id)
            .unwrap();
        let decoded_read_event = decoded_events
            .iter()
            .find(|e| e.id == read_event_id)
            .unwrap();

        assert_eq!(
            decoded_none_event,
            &DecodeEvent {
                source_crc: SOURCE_CRC,
                id: none_event_id,
                timestamp: Some(5),
                operations: vec![Operation {
                    objects: vec![ObjectIdentifier::new(5, None)],
                    address: Vec::new(),
                    read_data: Vec::new(),
                    write_data: Vec::new()
                }]
            }
        );
        assert_eq!(
            decoded_modify_event,
            &DecodeEvent {
                source_crc: SOURCE_CRC,
                id: modify_event_id,
                timestamp: Some(5),
                operations: vec![Operation {
                    objects: vec![ObjectIdentifier::new(5, None)],
                    address: vec![1],
                    read_data: vec![1, 2, 3],
                    write_data: vec![1, 2, 5],
                }]
            }
        );
        assert_eq!(
            decoded_read_event,
            &DecodeEvent {
                source_crc: SOURCE_CRC,
                id: read_event_id,
                timestamp: Some(5),
                operations: vec![Operation {
                    objects: vec![ObjectIdentifier::new(5, None)],
                    address: vec![12, 34, 56],
                    read_data: vec![1, 2, 3],
                    write_data: Vec::new(),
                }]
            }
        );
    }

    #[test]
    fn roundtrip_bulk_events() {
        static INTERFACE: TestTraceInterface = TestTraceInterface::new();
        INTERFACE.set_timestamp(Some(5));
        const SOURCE_CRC: u32 = 1234;

        let bulk_event = EncodeEvent::new(&INTERFACE, SOURCE_CRC);
        let bulk_event_id = bulk_event.id;
        let bulk_event = bulk_event.send_frame(&INTERFACE, |f| {
            f.send_object_identifier(&ObjectIdentifier::new(654, Some(-90)));
            f.send_object_identifier(&ObjectIdentifier::new(655, Some(1)));
            f.send_address(&[0xFF, 0xFF]);
            f.send_bytes(false, &[0xDD; 60]);
        });
        bulk_event.send_final_frame(&INTERFACE, |f| {
            f.send_bytes(true, &[0, 2, 3, 4]);
        });

        let written_bytes = INTERFACE.take_bytes();

        let mut decoder = StreamingDecoder::new();
        let decoded_events = decoder
            .push_bytes(&written_bytes)
            .into_iter()
            .map(|e| e.unwrap())
            .collect::<Vec<_>>();

        let bulk_event = decoded_events
            .iter()
            .find(|e| e.id == bulk_event_id)
            .unwrap();

        assert_eq!(
            bulk_event,
            &DecodeEvent {
                source_crc: SOURCE_CRC,
                id: bulk_event_id,
                timestamp: None,
                operations: vec![Operation {
                    objects: vec![
                        ObjectIdentifier::new(654, Some(-90)),
                        ObjectIdentifier::new(655, Some(1))
                    ],
                    address: vec![0xFF, 0xFF],
                    read_data: vec![0xDD; 60],
                    write_data: vec![0, 2, 3, 4],
                }]
            }
        );
    }
}
