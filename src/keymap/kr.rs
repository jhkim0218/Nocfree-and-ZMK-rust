use super::{Action, ansi};

pub const LAYOUT_ID: u8 = 3;
pub const LAYOUT_NAME: &str = "KR";
pub const PRODUCT_NAME: &str = "NocFree & KR";
pub const LEFT_KEY_COUNT: usize = 39;
pub const RIGHT_KEY_COUNT: usize = 50;
pub const KEY_COUNT: usize = LEFT_KEY_COUNT + RIGHT_KEY_COUNT;
pub const LEFT_FN_RAW: usize = 34;
pub const RIGHT_FN_RAW: usize = 80;
pub const EXPANDER_ADDRESSES: [u8; 4] = [0x20, 0x22, 0x24, 0x21];
pub const LEFT_ROW_COUNTS: [u8; 6] = [7, 7, 7, 7, 6, 5];
pub const RIGHT_ROW_COUNTS: [u8; 6] = [8, 8, 8, 7, 8, 7];
pub const EXTRA_LEFT_KEYS: usize = 0;
pub const EXTRA_RIGHT_KEYS: usize = 4;
pub const ROW_KEY_COUNTS: [usize; 6] = [17, 16, 16, 14, 14, 12];

#[rustfmt::skip]
pub const VISUAL_TO_RAW: [usize; KEY_COUNT] = [
    0, 1, 2, 3, 4, 5, 6, 39, 40, 41, 42, 43, 44, 45, 46, 85, 86,
    7, 8, 9, 10, 11, 12, 13, 47, 48, 49, 50, 51, 52, 53, 54, 87,
    14, 15, 16, 17, 18, 19, 20, 55, 56, 57, 58, 59, 60, 61, 62, 88,
    21, 22, 23, 24, 25, 26, 27, 63, 64, 65, 66, 67, 68, 69,
    28, 29, 30, 31, 32, 33, 70, 71, 72, 73, 74, 75, 76, 77,
    34, 35, 36, 37, 38, 78, 79, 80, 81, 82, 83, 84,
];

// 실측 raw가 교정되기 전에 같은 visual 위치가 사용하던 키 동작의 raw를 반환한다.
const fn action_source_raw(raw: usize) -> usize {
    match raw {
        39 => 86,
        40..=46 => raw - 1,
        85 => 46,
        86 => 70,
        47 => 87,
        48..=54 => raw - 1,
        87 => 54,
        55..=69 | 71..=77 => raw,
        88 => 78,
        70 => 88,
        78..=84 => raw + 1,
        _ => raw,
    }
}

// 교정 전 raw에 연결돼 있던 검증된 KR 키 동작을 반환한다.
const fn base_action_for_source_raw(raw: usize) -> Action {
    if raw < 20 {
        return ansi::base_action(raw);
    }
    match raw {
        20 => Action::Key(0x1c),
        21..=26 => ansi::base_action(raw - 1),
        27 => Action::Key(0x0b),
        28..=38 => ansi::base_action(raw - 2),
        39..=85 => ansi::base_action(ansi::LEFT_KEY_COUNT + raw - LEFT_KEY_COUNT),
        86 => Action::Key(0x3f),
        87 => Action::Key(0x23),
        88 => Action::Key(0x05),
        _ => panic!("raw key index outside the KR map"),
    }
}

// 실측한 물리 raw에 같은 visual 위치의 기존 키 동작을 연결한다.
pub const fn base_action(raw: usize) -> Action {
    base_action_for_source_raw(action_source_raw(raw))
}
