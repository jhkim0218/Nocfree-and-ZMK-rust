pub const SERVICE_UUID_LE: [u8; 16] = 0xf3641400_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();
pub const STATE_BYTES: usize = 8;
pub const COMMAND_BOOTLOADER: u8 = 1;
pub const CONNECTION_INTERVAL_UNITS: u16 = 6;
pub const CONNECTION_LATENCY: u16 = 30;
pub const CONNECTION_TIMEOUT_UNITS: u16 = 400;

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
    }
}
