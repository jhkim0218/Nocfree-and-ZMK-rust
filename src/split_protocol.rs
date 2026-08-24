pub const SERVICE_UUID_LE: [u8; 16] = 0xf3641400_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();
pub const STATE_BYTES: usize = 8;
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
        if matches!(data_type, 0x06 | 0x07)
            && value
                .chunks_exact(SERVICE_UUID_LE.len())
                .any(|uuid| uuid == SERVICE_UUID_LE)
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
    fn default_att_mtu_fits_every_split_value() {
        const ATT_HEADER_BYTES: usize = 3;
        const BACKLIGHT_BYTES: usize = 4;
        let value_capacity = SPLIT_ATT_MTU as usize - ATT_HEADER_BYTES;

        assert_eq!(SPLIT_ATT_MTU, 23);
        assert!(STATE_BYTES <= value_capacity);
        assert!(BACKLIGHT_BYTES <= value_capacity);
        assert!(core::mem::size_of::<u8>() <= value_capacity);
    }
}
