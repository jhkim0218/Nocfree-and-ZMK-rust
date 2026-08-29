use super::Action;

pub const LAYOUT_ID: u8 = 3;
pub const LAYOUT_NAME: &str = "KR";
pub const PRODUCT_NAME: &str = "NocFree & KR";
pub const LEFT_KEY_COUNT: usize = 39;
pub const RIGHT_KEY_COUNT: usize = 50;
pub const KEY_COUNT: usize = LEFT_KEY_COUNT + RIGHT_KEY_COUNT;
pub const LEFT_FN_RAW: usize = 34;
pub const RIGHT_FN_RAW: usize = 80;
pub const EXPANDER_ADDRESSES: [u8; 4] = [0x20, 0x22, 0x24, 0x21];
// KR physical scan report, 2026-08-28/29: 0x21 P0.0..P0.3 belongs only to Right.
pub const LEFT_ROW_COUNTS: [u8; 6] = [7, 7, 7, 7, 6, 5];
pub const RIGHT_ROW_COUNTS: [u8; 6] = [8, 8, 8, 7, 8, 7];
pub const EXTRA_LEFT_KEYS: usize = 0;
pub const EXTRA_RIGHT_KEYS: usize = 4;
pub const ROW_KEY_COUNTS: [usize; 6] = [17, 16, 16, 14, 14, 12];

macro_rules! k {
    ($usage:expr) => {
        Action::Key($usage)
    };
}

#[rustfmt::skip]
const VISUAL_BASE: [Action; KEY_COUNT] = [
    k!(0x29), k!(0x3a), k!(0x3b), k!(0x3c), k!(0x3d), k!(0x3e), k!(0x3f), k!(0x3f), k!(0x40), k!(0x41), k!(0x42), k!(0x43), k!(0x44), k!(0x45), k!(0x46), k!(0x4a), k!(0x4c),
    k!(0x35), k!(0x1e), k!(0x1f), k!(0x20), k!(0x21), k!(0x22), k!(0x23), k!(0x23), k!(0x24), k!(0x25), k!(0x26), k!(0x27), k!(0x2d), k!(0x2e), k!(0x2a), k!(0x4b), k!(0x2b),
    k!(0x14), k!(0x1a), k!(0x08), k!(0x15), k!(0x17), k!(0x1c), k!(0x1c), k!(0x18), k!(0x0c), k!(0x12), k!(0x13), k!(0x2f), k!(0x30), k!(0x31), k!(0x4e), k!(0x39), k!(0x04),
    k!(0x16), k!(0x07), k!(0x09), k!(0x0a), k!(0x0b), k!(0x0b), k!(0x0d), k!(0x0e), k!(0x0f), k!(0x33), k!(0x34), k!(0x28), k!(0xe1), k!(0x1d), k!(0x1b), k!(0x06), k!(0x19),
    k!(0x05), k!(0x05), k!(0x11), k!(0x10), k!(0x36), k!(0x37), k!(0x38), k!(0xe5), k!(0x52), Action::Fn, k!(0xe0), k!(0xe2), k!(0xe3), k!(0x2c), k!(0x2c), k!(0xe7), Action::Fn,
    k!(0xe6), k!(0x50), k!(0x51), k!(0x4f),
];

#[rustfmt::skip]
pub const VISUAL_TO_RAW: [usize; KEY_COUNT] = [
    0, 1, 2, 3, 4, 5, 6, 39, 40, 41, 42, 43, 44, 45, 46, 85, 86,
    7, 8, 9, 10, 11, 12, 13, 47, 48, 49, 50, 51, 52, 53, 54, 87,
    14, 15, 16, 17, 18, 19, 20, 55, 56, 57, 58, 59, 60, 61, 62, 88,
    21, 22, 23, 24, 25, 26, 27, 63, 64, 65, 66, 67, 68, 69,
    28, 29, 30, 31, 32, 33, 70, 71, 72, 73, 74, 75, 76, 77,
    34, 35, 36, 37, 38, 78, 79, 80, 81, 82, 83, 84,
];

pub const fn base_action(raw: usize) -> Action {
    let mut visual = 0;
    while visual < KEY_COUNT {
        if VISUAL_TO_RAW[visual] == raw {
            return VISUAL_BASE[visual];
        }
        visual += 1;
    }
    panic!("raw key index outside the KR map")
}
