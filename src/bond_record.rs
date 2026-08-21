pub const PROFILE_COUNT: usize = 5;
pub const SPLIT_BOND_SLOT: u8 = PROFILE_COUNT as u8;
pub const SYS_ATTR_CAPACITY: usize = 62;
pub const RECORD_BYTES: usize = 128;
pub const STORAGE_START: u32 = 0x65000;
pub const STORAGE_END: u32 = 0x6d000;
pub const PAGE_SIZE: u32 = 0x1000;
pub const SETTINGS_PAGE: u8 = PROFILE_COUNT as u8;
pub const SPLIT_PAGE: u8 = SETTINGS_PAGE + 1;

const BOND_MAGIC: [u8; 4] = *b"NFB1";
const SETTINGS_MAGIC: [u8; 4] = *b"NFS1";
const VERSION: u8 = 1;
const CRC_OFFSET: usize = 120;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BondRecord {
    pub master_ediv: u16,
    pub master_rand: [u8; 8],
    pub ltk: [u8; 16],
    pub encryption_flags: u8,
    pub irk: [u8; 16],
    pub address_flags: u8,
    pub address: [u8; 6],
    pub sys_attr_len: u8,
    pub sys_attrs: [u8; SYS_ATTR_CAPACITY],
}

pub fn encode_bond(profile: u8, record: &BondRecord) -> [u8; RECORD_BYTES] {
    assert!(profile <= SPLIT_BOND_SLOT);
    assert!((record.sys_attr_len as usize) <= SYS_ATTR_CAPACITY);
    let mut bytes = [0xff; RECORD_BYTES];
    bytes[0..4].copy_from_slice(&BOND_MAGIC);
    bytes[4] = VERSION;
    bytes[5] = profile;
    bytes[6] = record.sys_attr_len;
    bytes[8..10].copy_from_slice(&record.master_ediv.to_le_bytes());
    bytes[10..18].copy_from_slice(&record.master_rand);
    bytes[18..34].copy_from_slice(&record.ltk);
    bytes[34] = record.encryption_flags;
    bytes[35..51].copy_from_slice(&record.irk);
    bytes[51] = record.address_flags;
    bytes[52..58].copy_from_slice(&record.address);
    bytes[58..120].copy_from_slice(&record.sys_attrs);
    let crc = crc32(&bytes[..CRC_OFFSET]);
    bytes[CRC_OFFSET..CRC_OFFSET + 4].copy_from_slice(&crc.to_le_bytes());
    bytes
}

pub fn decode_bond(profile: u8, bytes: &[u8; RECORD_BYTES]) -> Option<BondRecord> {
    if bytes[0..4] != BOND_MAGIC
        || bytes[4] != VERSION
        || bytes[5] != profile
        || bytes[6] as usize > SYS_ATTR_CAPACITY
    {
        return None;
    }
    let stored_crc = u32::from_le_bytes(bytes[CRC_OFFSET..CRC_OFFSET + 4].try_into().ok()?);
    if stored_crc != crc32(&bytes[..CRC_OFFSET]) {
        return None;
    }

    Some(BondRecord {
        master_ediv: u16::from_le_bytes(bytes[8..10].try_into().ok()?),
        master_rand: bytes[10..18].try_into().ok()?,
        ltk: bytes[18..34].try_into().ok()?,
        encryption_flags: bytes[34],
        irk: bytes[35..51].try_into().ok()?,
        address_flags: bytes[51],
        address: bytes[52..58].try_into().ok()?,
        sys_attr_len: bytes[6],
        sys_attrs: bytes[58..120].try_into().ok()?,
    })
}

pub fn encode_selected_profile(profile: u8) -> [u8; RECORD_BYTES] {
    assert!((profile as usize) < PROFILE_COUNT);
    let mut bytes = [0xff; RECORD_BYTES];
    bytes[0..4].copy_from_slice(&SETTINGS_MAGIC);
    bytes[4] = VERSION;
    bytes[5] = profile;
    let crc = crc32(&bytes[..8]);
    bytes[8..12].copy_from_slice(&crc.to_le_bytes());
    bytes
}

pub fn decode_selected_profile(bytes: &[u8; RECORD_BYTES]) -> Option<u8> {
    if bytes[0..4] != SETTINGS_MAGIC || bytes[4] != VERSION || bytes[5] as usize >= PROFILE_COUNT {
        return None;
    }
    let stored_crc = u32::from_le_bytes(bytes[8..12].try_into().ok()?);
    (stored_crc == crc32(&bytes[..8])).then_some(bytes[5])
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffff_u32;
    for byte in bytes {
        crc ^= *byte as u32;
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(crc & 1);
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> BondRecord {
        BondRecord {
            master_ediv: 0x1234,
            master_rand: [1; 8],
            ltk: [2; 16],
            encryption_flags: 3,
            irk: [4; 16],
            address_flags: 5,
            address: [6; 6],
            sys_attr_len: 7,
            sys_attrs: [8; SYS_ATTR_CAPACITY],
        }
    }

    #[test]
    fn bond_record_round_trips() {
        let record = sample();
        assert_eq!(decode_bond(3, &encode_bond(3, &record)), Some(record));
        assert_eq!(
            decode_bond(SPLIT_BOND_SLOT, &encode_bond(SPLIT_BOND_SLOT, &record)),
            Some(record)
        );
    }

    #[test]
    fn wrong_profile_and_corruption_are_rejected() {
        let mut encoded = encode_bond(2, &sample());
        assert_eq!(decode_bond(1, &encoded), None);
        encoded[40] ^= 1;
        assert_eq!(decode_bond(2, &encoded), None);
        assert_eq!(decode_bond(2, &[0xff; RECORD_BYTES]), None);
    }

    #[test]
    fn selected_profile_round_trips_and_checks_crc() {
        let mut encoded = encode_selected_profile(4);
        assert_eq!(decode_selected_profile(&encoded), Some(4));
        encoded[5] = 3;
        assert_eq!(decode_selected_profile(&encoded), None);
    }

    #[test]
    fn bond_pages_stay_below_factory_flash() {
        assert_eq!(STORAGE_START + SETTINGS_PAGE as u32 * PAGE_SIZE, 0x6a000);
        assert_eq!(STORAGE_START + SPLIT_PAGE as u32 * PAGE_SIZE, 0x6b000);
        assert_eq!(STORAGE_START + (SPLIT_PAGE as u32 + 1) * PAGE_SIZE, 0x6c000);
        assert!(STORAGE_START + (SPLIT_PAGE as u32 + 1) * PAGE_SIZE < STORAGE_END);
    }
}
