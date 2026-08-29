#[cfg(target_arch = "arm")]
use core::cell::RefCell;

#[cfg(target_arch = "arm")]
use embassy_sync::blocking_mutex::Mutex;
#[cfg(target_arch = "arm")]
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
#[cfg(target_arch = "arm")]
use embassy_time::Instant;

pub const DIAGNOSTIC_CAPACITY: usize = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SplitDiagnosticRole {
    Left = b'L',
    Right = b'R',
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SplitDiagnosticEvent {
    ScanStart = 1,
    AdvertisementFound = 2,
    ScanError = 3,
    ConnectStart = 4,
    Connected = 5,
    ConnectError = 6,
    ConnectionParameters = 7,
    SecurityStart = 8,
    SecurityOk = 9,
    SecurityError = 10,
    GattStart = 11,
    GattOk = 12,
    GattError = 13,
    SplitReady = 14,
    Disconnected = 15,
    Advertising = 16,
    AdvertisingError = 17,
    DisconnectedKey = 18,
    KeyScanConfigured = 19,
    KeyScanInputs = 20,
    BacklightPwm = 21,
}

pub fn pack_address(flags: u8, bytes: [u8; 6]) -> u64 {
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], flags, 0,
    ])
}

pub fn pack_connection_parameters(
    min_interval: u16,
    max_interval: u16,
    latency: u16,
    timeout: u16,
) -> u64 {
    u64::from_le_bytes([
        min_interval as u8,
        (min_interval >> 8) as u8,
        max_interval as u8,
        (max_interval >> 8) as u8,
        latency as u8,
        (latency >> 8) as u8,
        timeout as u8,
        (timeout >> 8) as u8,
    ])
}

// 최대 네 PCA9555 입력값을 CDC 진단 레코드의 64비트 데이터 필드에 보존한다.
pub const fn pack_key_scan_words<const N: usize>(words: [u16; N]) -> u64 {
    let mut packed = 0_u64;
    let mut index = 0;
    while index < N && index < 4 {
        packed |= (words[index] as u64) << (index * 16);
        index += 1;
    }
    packed
}

pub fn duration_millis(start_ms: u64, end_ms: u64) -> u16 {
    end_ms.saturating_sub(start_ms).min(u16::MAX as u64) as u16
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SplitDiagnosticRecord {
    pub data: u64,
    pub timestamp_ms: u32,
    pub value: u16,
    pub arg: i8,
    pub event: SplitDiagnosticEvent,
}

const EMPTY_RECORD: SplitDiagnosticRecord = SplitDiagnosticRecord {
    data: 0,
    timestamp_ms: 0,
    value: 0,
    arg: 0,
    event: SplitDiagnosticEvent::ScanStart,
};

#[derive(Clone, Copy)]
pub struct SplitDiagnosticLog<const N: usize> {
    records: [SplitDiagnosticRecord; N],
    next: usize,
    len: usize,
    dropped: u32,
}

impl<const N: usize> Default for SplitDiagnosticLog<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> SplitDiagnosticLog<N> {
    pub const fn new() -> Self {
        assert!(N > 0);
        Self {
            records: [EMPTY_RECORD; N],
            next: 0,
            len: 0,
            dropped: 0,
        }
    }

    pub fn push(&mut self, record: SplitDiagnosticRecord) {
        if self.len == N {
            self.dropped = self.dropped.saturating_add(1);
        } else {
            self.len += 1;
        }
        self.records[self.next] = record;
        self.next = (self.next + 1) % N;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn dropped(&self) -> u32 {
        self.dropped
    }

    pub fn iter(&self) -> impl Iterator<Item = &SplitDiagnosticRecord> {
        let oldest = if self.len == N { self.next } else { 0 };
        (0..self.len).map(move |offset| &self.records[(oldest + offset) % N])
    }
}

#[cfg(target_arch = "arm")]
pub struct SplitDiagnostics<const N: usize> {
    log: Mutex<CriticalSectionRawMutex, RefCell<SplitDiagnosticLog<N>>>,
}

#[cfg(target_arch = "arm")]
impl<const N: usize> Default for SplitDiagnostics<N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_arch = "arm")]
impl<const N: usize> SplitDiagnostics<N> {
    pub const fn new() -> Self {
        Self {
            log: Mutex::new(RefCell::new(SplitDiagnosticLog::new())),
        }
    }

    pub fn record(&self, event: SplitDiagnosticEvent, arg: i8, value: u16, data: u64) {
        let timestamp_ms = Instant::now().as_millis().min(u32::MAX as u64) as u32;
        self.log.lock(|log| {
            log.borrow_mut().push(SplitDiagnosticRecord {
                data,
                timestamp_ms,
                value,
                arg,
                event,
            });
        });
    }

    pub fn snapshot(&self) -> SplitDiagnosticLog<N> {
        self.log.lock(|log| *log.borrow())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(timestamp_ms: u32) -> SplitDiagnosticRecord {
        SplitDiagnosticRecord {
            data: timestamp_ms as u64,
            timestamp_ms,
            value: timestamp_ms as u16,
            arg: timestamp_ms as i8,
            event: SplitDiagnosticEvent::Connected,
        }
    }

    #[test]
    fn keeps_recent_records_in_chronological_order() {
        let mut log = SplitDiagnosticLog::<3>::new();
        for timestamp in 1..=5 {
            log.push(record(timestamp));
        }

        let timestamps: Vec<_> = log.iter().map(|record| record.timestamp_ms).collect();
        assert_eq!(timestamps, [3, 4, 5]);
        assert_eq!(log.len(), 3);
        assert_eq!(log.dropped(), 2);
    }

    #[test]
    fn packs_identity_and_connection_parameters_without_loss() {
        assert_eq!(
            pack_address(3, [1, 2, 3, 4, 5, 6]).to_le_bytes(),
            [1, 2, 3, 4, 5, 6, 3, 0]
        );
        assert_eq!(
            pack_connection_parameters(6, 6, 30, 400).to_le_bytes(),
            [6, 0, 6, 0, 30, 0, 144, 1]
        );
        assert_eq!(duration_millis(100, 175), 75);
        assert_eq!(duration_millis(200, 100), 0);
    }

    #[test]
    fn packs_key_scan_words_without_loss() {
        // 이 테스트가 검증하는 시나리오: 네 확장기의 원시 입력값을 CDC 진단 한 레코드에 손실 없이 담는다.
        // Given: 서로 구분되는 네 개의 16비트 입력값이 있다.
        let words = [0x0123, 0x4567, 0x89ab, 0xcdef];

        // When: 진단용 64비트 값으로 묶는다.
        let packed = pack_key_scan_words(words);

        // Then: 바이트 순서와 모든 입력 비트가 그대로 유지된다.
        assert_eq!(
            packed.to_le_bytes(),
            [0x23, 0x01, 0x67, 0x45, 0xab, 0x89, 0xef, 0xcd]
        );
        assert_eq!(
            pack_key_scan_words([0x0123, 0x4567, 0x89ab]).to_le_bytes(),
            [0x23, 0x01, 0x67, 0x45, 0xab, 0x89, 0, 0]
        );
    }
}
