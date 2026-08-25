use super::{Action, ansi};

pub const LAYOUT_ID: u8 = 3;
pub const LAYOUT_NAME: &str = "KR";
pub const PRODUCT_NAME: &str = "NocFree & KR";
pub const LEFT_KEY_COUNT: usize = 39;
pub const RIGHT_KEY_COUNT: usize = 50;
pub const KEY_COUNT: usize = LEFT_KEY_COUNT + RIGHT_KEY_COUNT;
pub const LEFT_FN_RAW: usize = 32;
pub const RIGHT_FN_RAW: usize = 81;
pub const EXPANDER_ADDRESSES: [u8; 4] = [0x20, 0x22, 0x24, 0x21];
pub const LEFT_ROW_COUNTS: [u8; 6] = ansi::LEFT_ROW_COUNTS;
pub const RIGHT_ROW_COUNTS: [u8; 6] = ansi::RIGHT_ROW_COUNTS;
pub const EXTRA_LEFT_KEYS: usize = 2;
pub const EXTRA_RIGHT_KEYS: usize = 3;
pub const ROW_KEY_COUNTS: [usize; 6] = [17, 16, 16, 14, 14, 12];

#[rustfmt::skip]
pub const VISUAL_TO_RAW: [usize; KEY_COUNT] = [
    0, 1, 2, 3, 4, 5, 6, 86, 39, 40, 41, 42, 43, 44, 45, 46, 70,
    7, 8, 9, 10, 11, 12, 13, 87, 47, 48, 49, 50, 51, 52, 53, 54,
    14, 15, 16, 17, 18, 19, 37, 55, 56, 57, 58, 59, 60, 61, 62, 78,
    20, 21, 22, 23, 24, 25, 38, 63, 64, 65, 66, 67, 68, 69,
    26, 27, 28, 29, 30, 31, 88, 71, 72, 73, 74, 75, 76, 77,
    32, 33, 34, 35, 36, 79, 80, 81, 82, 83, 84, 85,
];

pub const fn base_action(raw: usize) -> Action {
    if raw < ansi::LEFT_KEY_COUNT {
        return ansi::base_action(raw);
    }
    match raw {
        37 => Action::Key(0x1c),
        38 => Action::Key(0x0b),
        39..=85 => ansi::base_action(ansi::LEFT_KEY_COUNT + raw - LEFT_KEY_COUNT),
        86 => Action::Key(0x3f),
        87 => Action::Key(0x23),
        88 => Action::Key(0x05),
        _ => panic!("raw key index outside the KR map"),
    }
}
