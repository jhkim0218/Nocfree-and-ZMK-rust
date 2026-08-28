pub const SERVICE_UUID_LE: [u8; 16] = 0xf3641400_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();
pub const STATE_BYTES: usize = 20;
pub const CLOCK_BYTES: usize = 16;
pub const STATE_FLAG_RECONCILE: u8 = 1;
pub const CLOCK_SYNC_SAMPLES: usize = 3;
pub const CLOCK_SYNC_REFRESH_SECS: u64 = 60;
pub const COMMAND_BOOTLOADER: u8 = 1;
pub const COMMAND_BATTERY_REQUEST: u8 = 5;
pub const CONNECTION_INTERVAL_UNITS: u16 = 6;
pub const CONNECTION_LATENCY: u16 = 30;
pub const CONNECTION_TIMEOUT_UNITS: u16 = 400;
pub const SPLIT_ATT_MTU: u16 = 23;
pub const FAST_ADVERTISING_INTERVAL_UNITS: u32 = 400;
pub const MEDIUM_ADVERTISING_INTERVAL_UNITS: u32 = 800;
pub const IDLE_ADVERTISING_INTERVAL_UNITS: u32 = 1_600;
pub const FAST_ADVERTISING_DURATION_UNITS: u16 = 1_000;
pub const MEDIUM_ADVERTISING_DURATION_UNITS: u16 = 5_000;
pub const RIGHT_SPLIT_TX_POWER_DBM: i8 = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SplitStateFrame {
    pub pressed: u64,
    pub source_micros: u64,
    pub sequence: u16,
    pub flags: u8,
}

impl SplitStateFrame {
    pub const fn encode(self) -> [u8; STATE_BYTES] {
        let pressed = self.pressed.to_le_bytes();
        let source_micros = self.source_micros.to_le_bytes();
        let sequence = self.sequence.to_le_bytes();
        [
            pressed[0],
            pressed[1],
            pressed[2],
            pressed[3],
            pressed[4],
            pressed[5],
            pressed[6],
            pressed[7],
            source_micros[0],
            source_micros[1],
            source_micros[2],
            source_micros[3],
            source_micros[4],
            source_micros[5],
            source_micros[6],
            source_micros[7],
            sequence[0],
            sequence[1],
            self.flags,
            0,
        ]
    }

    pub fn decode(bytes: [u8; STATE_BYTES]) -> Option<Self> {
        (bytes[19] == 0).then(|| Self {
            pressed: u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
            source_micros: u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
            sequence: u16::from_le_bytes(bytes[16..18].try_into().unwrap()),
            flags: bytes[18],
        })
    }

    pub const fn reconcile(self) -> bool {
        self.flags & STATE_FLAG_RECONCILE != 0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClockSample {
    pub offset_micros: i64,
    pub round_trip_micros: u64,
}

impl ClockSample {
    pub fn estimate(
        left_sent_micros: u64,
        echoed_left_micros: u64,
        right_received_micros: u64,
        left_received_micros: u64,
    ) -> Option<Self> {
        if echoed_left_micros != left_sent_micros || left_received_micros < left_sent_micros {
            return None;
        }
        let round_trip_micros = left_received_micros - left_sent_micros;
        let midpoint = left_sent_micros + round_trip_micros / 2;
        let offset_micros = if midpoint >= right_received_micros {
            i64::try_from(midpoint - right_received_micros).ok()?
        } else {
            -i64::try_from(right_received_micros - midpoint).ok()?
        };
        Some(Self {
            offset_micros,
            round_trip_micros,
        })
    }

    pub const fn right_to_left(self, right_micros: u64) -> u64 {
        if self.offset_micros >= 0 {
            right_micros.saturating_add(self.offset_micros as u64)
        } else {
            right_micros.saturating_sub(self.offset_micros.unsigned_abs())
        }
    }
}

pub const fn clock_request(left_sent_micros: u64) -> [u8; CLOCK_BYTES] {
    let left = left_sent_micros.to_le_bytes();
    [
        left[0], left[1], left[2], left[3], left[4], left[5], left[6], left[7], 0, 0, 0, 0, 0, 0,
        0, 0,
    ]
}

pub fn clock_response(request: [u8; CLOCK_BYTES], right_received_micros: u64) -> [u8; CLOCK_BYTES] {
    let mut response = request;
    response[8..16].copy_from_slice(&right_received_micros.to_le_bytes());
    response
}

pub fn decode_clock_response(bytes: [u8; CLOCK_BYTES]) -> (u64, u64) {
    (
        u64::from_le_bytes(bytes[0..8].try_into().unwrap()),
        u64::from_le_bytes(bytes[8..16].try_into().unwrap()),
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(i8)]
pub enum AdvertisingStage {
    Fast = 0,
    Medium = 1,
    Idle = 2,
}

impl AdvertisingStage {
    pub const fn interval(self) -> u32 {
        match self {
            Self::Fast => FAST_ADVERTISING_INTERVAL_UNITS,
            Self::Medium => MEDIUM_ADVERTISING_INTERVAL_UNITS,
            Self::Idle => IDLE_ADVERTISING_INTERVAL_UNITS,
        }
    }

    pub const fn timeout(self) -> Option<u16> {
        match self {
            Self::Fast => Some(FAST_ADVERTISING_DURATION_UNITS),
            Self::Medium => Some(MEDIUM_ADVERTISING_DURATION_UNITS),
            Self::Idle => None,
        }
    }

    pub const fn next(self) -> Self {
        match self {
            Self::Fast => Self::Medium,
            Self::Medium | Self::Idle => Self::Idle,
        }
    }
}

pub fn advertisement_has_split_service(mut data: &[u8]) -> bool {
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

    #[test]
    fn finds_the_split_uuid_in_a_valid_advertisement() {
        let mut data = [0_u8; 21];
        data[0..3].copy_from_slice(&[2, 0x01, 0x06]);
        data[3] = 17;
        data[4] = 0x07;
        data[5..].copy_from_slice(&SERVICE_UUID_LE);
        assert!(advertisement_has_split_service(&data));
    }

    #[test]
    fn rejects_other_and_malformed_advertisements() {
        assert!(!advertisement_has_split_service(&[3, 0x07, 1]));
        assert!(!advertisement_has_split_service(&[2, 0x01, 0x06]));
        assert!(!advertisement_has_split_service(&[0]));
    }

    #[test]
    fn split_timing_matches_zmk_defaults() {
        assert_eq!(CONNECTION_INTERVAL_UNITS, 6);
        assert_eq!(CONNECTION_LATENCY, 30);
        assert_eq!(CONNECTION_TIMEOUT_UNITS, 400);
        assert_eq!(FAST_ADVERTISING_INTERVAL_UNITS, 400);
        assert_eq!(MEDIUM_ADVERTISING_INTERVAL_UNITS, 800);
        assert_eq!(IDLE_ADVERTISING_INTERVAL_UNITS, 1_600);
    }

    #[test]
    fn disconnected_advertising_slows_down_in_order() {
        assert_eq!(AdvertisingStage::Fast.interval(), 400);
        assert_eq!(AdvertisingStage::Fast.timeout(), Some(1_000));
        assert_eq!(AdvertisingStage::Fast.next(), AdvertisingStage::Medium);
        assert_eq!(AdvertisingStage::Medium.interval(), 800);
        assert_eq!(AdvertisingStage::Medium.timeout(), Some(5_000));
        assert_eq!(AdvertisingStage::Medium.next(), AdvertisingStage::Idle);
        assert_eq!(AdvertisingStage::Idle.interval(), 1_600);
        assert_eq!(AdvertisingStage::Idle.timeout(), None);
        assert_eq!(AdvertisingStage::Idle.next(), AdvertisingStage::Idle);
    }

    #[test]
    fn right_split_uses_the_nrf52833_max_tx_power() {
        assert_eq!(RIGHT_SPLIT_TX_POWER_DBM, 8);
    }

    #[test]
    fn state_frame_round_trips_in_one_default_att_value() {
        let frame = SplitStateFrame {
            pressed: 0x0123_4567_89ab_cdef,
            source_micros: 0xfedc_ba98_7654_3210,
            sequence: u16::MAX,
            flags: STATE_FLAG_RECONCILE,
        };
        assert_eq!(SplitStateFrame::decode(frame.encode()), Some(frame));
        assert!(frame.reconcile());
    }

    #[test]
    fn clock_exchange_estimates_offset_and_rejects_wrong_echo() {
        let response = clock_response(clock_request(10_000), 7_250);
        let (echoed, right_received) = decode_clock_response(response);
        let sample = ClockSample::estimate(10_000, echoed, right_received, 10_500).unwrap();
        assert_eq!(sample.round_trip_micros, 500);
        assert_eq!(sample.offset_micros, 3_000);
        assert_eq!(sample.right_to_left(8_000), 11_000);
        assert!(ClockSample::estimate(10_000, 9_999, right_received, 10_500).is_none());
    }

    #[test]
    fn default_att_mtu_fits_every_split_value() {
        const ATT_HEADER_BYTES: usize = 3;
        const BACKLIGHT_BYTES: usize = 4;
        let value_capacity = SPLIT_ATT_MTU as usize - ATT_HEADER_BYTES;

        assert_eq!(SPLIT_ATT_MTU, 23);
        assert_eq!(STATE_BYTES, value_capacity);
        assert!(CLOCK_BYTES <= value_capacity);
        assert!(BACKLIGHT_BYTES <= value_capacity);
        assert!(core::mem::size_of::<u8>() <= value_capacity);
    }
}
