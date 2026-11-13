#![cfg_attr(not(any(test, feature = "std")), no_std)]

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

pub struct TraceInterface {
    /// Acquire global mutable access to the interface.
    /// This must not be reentrant.
    pub acquire: fn(),
    /// Write data into the sink
    pub write: unsafe fn(buf: &[u8]),
    /// Get the current time in microseconds.
    /// If a timestamp is not desired, return None.
    pub timestamp: unsafe fn() -> Option<u64>,
    /// Release the global mutable access to the interface.
    pub release: unsafe fn(),
    /// Get a new id for trace events.
    /// Should increment by 1 every time when called and wrap around to 0 at the end.
    pub id: fn() -> u8,
}

pub fn crc32(data: &[u8]) -> u32 {
    const POLY: u32 = 0x82f6_3b78; // Castagnoli

    let mut crc = 0xFFFF_FFFF;

    for byte in data {
        crc ^= u32::from(*byte);

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

#[cfg(test)]
mod tests {
    use core::sync::atomic::{AtomicU8, Ordering};

    use critical_section::RestoreState;

    use crate::{
        decoder::{Event as DecodeEvent, Operation, StreamingDecoder},
        encoder::Event as EncodeEvent,
    };

    use super::*;

    #[test]
    fn roundtrip_single_events() {
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
            timestamp: || Some(5),
            release: || unsafe {
                TAKEN = false;
                critical_section::release(RESTORE_STATE);
            },
            id: || {
                static ID: AtomicU8 = AtomicU8::new(0);
                ID.fetch_add(1, Ordering::Relaxed)
            },
        };
        const SOURCE_CRC: u32 = 54321;

        let none_event = EncodeEvent::new(&INTERFACE, SOURCE_CRC);
        let none_event_id = none_event.id;
        none_event.send_final_frame(&INTERFACE, |f| {
            f.send_timestamp();
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
        });

        let modify_event = EncodeEvent::new(&INTERFACE, SOURCE_CRC);
        let modify_event_id = modify_event.id;
        let modify_event = modify_event.send_frame(&INTERFACE, |f| {
            f.send_timestamp();
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
            f.send_timestamp();
            f.send_object_identifier(&ObjectIdentifier::new(5, None));
            f.send_address(&[12, 34, 56]);
            f.send_bytes(false, &[1, 2, 3]);
        });

        (INTERFACE.acquire)();
        #[allow(static_mut_refs)]
        let written_bytes = unsafe { WRITTEN_BYTES.clone() };
        unsafe {
            (INTERFACE.release)();
        }

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
            timestamp: || Some(5),
            release: || unsafe {
                TAKEN = false;
                critical_section::release(RESTORE_STATE);
            },
            id: || {
                static ID: AtomicU8 = AtomicU8::new(0);
                ID.fetch_add(1, Ordering::Relaxed)
            },
        };
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

        (INTERFACE.acquire)();
        #[allow(static_mut_refs)]
        let written_bytes = unsafe { WRITTEN_BYTES.clone() };
        unsafe {
            (INTERFACE.release)();
        }

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
