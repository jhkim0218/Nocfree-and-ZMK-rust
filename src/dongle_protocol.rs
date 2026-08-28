use crate::report::{KEY_BITMAP_BYTES, KEYBOARD_REPORT_BYTES, KeyboardReport, ReportFrame};

pub const SERVICE_UUID_LE: [u8; 16] = 0xf3641500_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();
pub const UNIVERSAL_KEYBOARD_REPORT_BYTES: usize = 19;
pub const REPORT_BYTES: usize = 2 + UNIVERSAL_KEYBOARD_REPORT_BYTES + 2;
pub const ATT_MTU: u16 = REPORT_BYTES as u16 + 3;
pub const CONNECTION_INTERVAL_UNITS: u16 = 6;
pub const CONNECTION_LATENCY: u16 = 0;
pub const CONNECTION_TIMEOUT_UNITS: u16 = 400;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DongleReport {
    pub sequence: u16,
    pub frame: ReportFrame,
}

impl DongleReport {
    pub fn encode(self) -> [u8; REPORT_BYTES] {
        let mut bytes = [0; REPORT_BYTES];
        bytes[..2].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[2..2 + KEYBOARD_REPORT_BYTES].copy_from_slice(self.frame.keyboard.as_bytes());
        bytes[2 + UNIVERSAL_KEYBOARD_REPORT_BYTES..]
            .copy_from_slice(&self.frame.consumer.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: [u8; REPORT_BYTES]) -> Self {
        Self {
            sequence: u16::from_le_bytes(bytes[..2].try_into().unwrap()),
            frame: ReportFrame {
                keyboard: KeyboardReport {
                    modifiers: bytes[2],
                    reserved: bytes[3],
                    keys: bytes[4..4 + KEY_BITMAP_BYTES].try_into().unwrap(),
                },
                consumer: u16::from_le_bytes(
                    bytes[2 + UNIVERSAL_KEYBOARD_REPORT_BYTES..]
                        .try_into()
                        .unwrap(),
                ),
            },
        }
    }
}

pub struct DongleReportReceiver {
    last_sequence: Option<u16>,
}

impl DongleReportReceiver {
    pub const fn new() -> Self {
        Self {
            last_sequence: None,
        }
    }

    pub fn accept(&mut self, bytes: [u8; REPORT_BYTES]) -> Option<ReportFrame> {
        let report = DongleReport::decode(bytes);
        let current = report.sequence;
        if self.last_sequence.is_some_and(|last| {
            let distance = current.wrapping_sub(last);
            distance == 0 || distance >= 0x8000
        }) {
            return None;
        }
        self.last_sequence = Some(current);
        Some(report.frame)
    }
}

impl Default for DongleReportReceiver {
    fn default() -> Self {
        Self::new()
    }
}

pub fn advertisement_has_dongle_service(mut data: &[u8]) -> bool {
    while let Some((&length, rest)) = data.split_first() {
        let length = length as usize;
        if length == 0 || rest.len() < length {
            return false;
        }
        let data_type = rest[0];
        let value = &rest[1..length];
        if matches!(data_type, 0x06 | 0x07) && value.as_chunks::<16>().0.contains(&SERVICE_UUID_LE)
        {
            return true;
        }
        data = &rest[length..];
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    fn frame(modifiers: u8, key: u8, consumer: u16) -> ReportFrame {
        let mut keys = [0; KEY_BITMAP_BYTES];
        keys[0] = key;
        ReportFrame {
            keyboard: KeyboardReport {
                modifiers,
                reserved: 0,
                keys,
            },
            consumer,
        }
    }

    #[test]
    fn report_round_trips_every_hid_field() {
        let report = DongleReport {
            sequence: 0x1234,
            frame: frame(5, 9, 0x02a0),
        };
        assert_eq!(DongleReport::decode(report.encode()), report);
        assert_eq!(REPORT_BYTES, UNIVERSAL_KEYBOARD_REPORT_BYTES + 4);
        assert!(
            report.encode()[2 + KEYBOARD_REPORT_BYTES..2 + UNIVERSAL_KEYBOARD_REPORT_BYTES]
                .iter()
                .all(|byte| *byte == 0)
        );
        assert!(ATT_MTU <= 64);
    }

    #[test]
    fn receiver_rejects_duplicates_and_old_frames_but_accepts_wrap() {
        let mut receiver = DongleReportReceiver::new();
        let encoded = |sequence| {
            DongleReport {
                sequence,
                frame: frame(0, sequence as u8, 0),
            }
            .encode()
        };
        assert!(receiver.accept(encoded(0xfffe)).is_some());
        assert!(receiver.accept(encoded(0xfffe)).is_none());
        assert!(receiver.accept(encoded(0xfffd)).is_none());
        assert!(receiver.accept(encoded(0xffff)).is_some());
        assert!(receiver.accept(encoded(0)).is_some());
    }

    #[test]
    fn advertisement_requires_the_complete_dongle_uuid() {
        let mut data = [0_u8; 21];
        data[..3].copy_from_slice(&[2, 0x01, 0x06]);
        data[3] = 17;
        data[4] = 0x07;
        data[5..].copy_from_slice(&SERVICE_UUID_LE);
        assert!(advertisement_has_dongle_service(&data));
        data[20] ^= 1;
        assert!(!advertisement_has_dongle_service(&data));
        assert!(!advertisement_has_dongle_service(&[17, 0x07, 1]));
    }
}
