#[cfg(any(
    all(feature = "layout-ansi", feature = "layout-iso"),
    all(feature = "layout-ansi", feature = "layout-jis"),
    all(feature = "layout-ansi", feature = "layout-kr"),
    all(feature = "layout-iso", feature = "layout-jis"),
    all(feature = "layout-iso", feature = "layout-kr"),
    all(feature = "layout-jis", feature = "layout-kr"),
))]
compile_error!("select exactly one of layout-ansi, layout-iso, layout-jis, or layout-kr");

#[cfg(not(any(
    feature = "layout-ansi",
    feature = "layout-iso",
    feature = "layout-jis",
    feature = "layout-kr"
)))]
compile_error!("select one of layout-ansi, layout-iso, layout-jis, or layout-kr");

#[cfg_attr(not(feature = "layout-ansi"), allow(dead_code))]
mod ansi;
#[cfg(feature = "layout-iso")]
mod iso;
#[cfg(feature = "layout-jis")]
mod jis;
#[cfg(feature = "layout-kr")]
mod kr;

#[cfg(feature = "layout-ansi")]
use ansi as selected;
#[cfg(feature = "layout-iso")]
use iso as selected;
#[cfg(feature = "layout-jis")]
use jis as selected;
#[cfg(feature = "layout-kr")]
use kr as selected;

pub use selected::{
    EXPANDER_ADDRESSES, EXTRA_LEFT_KEYS, EXTRA_RIGHT_KEYS, KEY_COUNT, LAYOUT_ID, LAYOUT_NAME,
    LEFT_FN_RAW, LEFT_KEY_COUNT, LEFT_ROW_COUNTS, PRODUCT_NAME, RIGHT_FN_RAW, RIGHT_KEY_COUNT,
    RIGHT_ROW_COUNTS, ROW_KEY_COUNTS, VISUAL_TO_RAW,
};

pub const EXPANDER_COUNT: usize = EXPANDER_ADDRESSES.len();
pub const MATRIX_ROWS: usize = ROW_KEY_COUNTS.len();
pub const MATRIX_COLS: usize = 21;
pub const MAX_HALF_KEY_COUNT: usize = if LEFT_KEY_COUNT > RIGHT_KEY_COUNT {
    LEFT_KEY_COUNT
} else {
    RIGHT_KEY_COUNT
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Action {
    #[default]
    Transparent,
    NoAction,
    Fn,
    Key(u8),
    Chord {
        modifiers: u8,
        key: u8,
    },
    Consumer(u16),
    LayerMomentary(u8),
    LayerToggle(u8),
    ResetLeft,
    BootLeft,
    BootRight,
    ProfileSelect(u8),
    ProfileShortcut(u8),
    ProfileClear,
    OutputUsb,
    OutputBle,
    BacklightToggle,
    BacklightDown,
    BacklightUp,
    BatteryStatus,
    SystemShortcut {
        system: u8,
        key: u8,
    },
    SystemF3,
    SystemF4,
}

pub const fn base_action(raw: usize) -> Action {
    selected::base_action(raw)
}

pub const fn function_action(raw: usize) -> Action {
    function_action_for(base_action(raw))
}

pub const fn raw_to_visual(raw: usize) -> usize {
    let mut visual = 0;
    while visual < KEY_COUNT {
        if VISUAL_TO_RAW[visual] == raw {
            return visual;
        }
        visual += 1;
    }
    panic!("raw key index outside the selected layout")
}

pub const fn raw_from_matrix(row: usize, col: usize) -> Option<usize> {
    if row >= MATRIX_ROWS || col >= ROW_KEY_COUNTS[row] {
        return None;
    }
    let mut visual = col;
    let mut preceding = 0;
    while preceding < row {
        visual += ROW_KEY_COUNTS[preceding];
        preceding += 1;
    }
    Some(VISUAL_TO_RAW[visual])
}

pub const fn matrix_from_raw(raw: usize) -> Option<(usize, usize)> {
    if raw >= KEY_COUNT {
        return None;
    }
    let visual = raw_to_visual(raw);
    let mut row = 0;
    let mut start = 0;
    while row < MATRIX_ROWS {
        let end = start + ROW_KEY_COUNTS[row];
        if visual < end {
            return Some((row, visual - start));
        }
        start = end;
        row += 1;
    }
    None
}

const fn function_action_for(base: Action) -> Action {
    match base {
        Action::Key(0x29) => Action::ResetLeft,
        Action::Key(0x3a) => Action::Consumer(0x006f),
        Action::Key(0x3b) => Action::Consumer(0x0070),
        Action::Key(0x3c) => Action::SystemF3,
        Action::Key(0x3d) => Action::SystemF4,
        Action::Key(0x3e) => Action::BacklightDown,
        Action::Key(0x3f) => Action::BacklightUp,
        Action::Key(0x40) => Action::Consumer(0x00b6),
        Action::Key(0x41) => Action::Consumer(0x00cd),
        Action::Key(0x42) => Action::Consumer(0x00b5),
        Action::Key(0x43) => Action::Consumer(0x00e2),
        Action::Key(0x44) => Action::Consumer(0x00ea),
        Action::Key(0x45) => Action::Consumer(0x00e9),
        Action::Key(0x1e) => Action::ProfileShortcut(0),
        Action::Key(0x1f) => Action::ProfileShortcut(1),
        Action::Key(0x20) => Action::ProfileShortcut(2),
        Action::Key(0x22) => Action::BootLeft,
        Action::Key(0x27) | Action::Key(0x4c) => Action::BootRight,
        Action::Key(0x2b) => Action::BacklightToggle,
        Action::Key(0x0c) => Action::BatteryStatus,
        Action::Key(0x11) => Action::SystemShortcut {
            system: 1,
            key: 0x11,
        },
        Action::Key(0x10) => Action::SystemShortcut {
            system: 0,
            key: 0x10,
        },
        _ => Action::Transparent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_transform_is_a_permutation() {
        let mut seen = [false; KEY_COUNT];
        for raw in 0..KEY_COUNT {
            let visual = raw_to_visual(raw);
            assert!(!seen[visual], "duplicate visual index {visual}");
            seen[visual] = true;
            assert_eq!(
                raw_from_matrix(
                    matrix_from_raw(raw).unwrap().0,
                    matrix_from_raw(raw).unwrap().1
                ),
                Some(raw)
            );
        }
        assert!(seen.into_iter().all(|value| value));
        assert_eq!(ROW_KEY_COUNTS.iter().sum::<usize>(), KEY_COUNT);
    }

    #[test]
    fn halves_and_function_keys_are_not_swapped() {
        assert!(LEFT_FN_RAW < LEFT_KEY_COUNT);
        assert!(RIGHT_FN_RAW >= LEFT_KEY_COUNT);
        assert_eq!(base_action(LEFT_FN_RAW), Action::Fn);
        assert_eq!(base_action(RIGHT_FN_RAW), Action::Fn);
        assert_eq!(function_action(0), Action::ResetLeft);
    }

    #[test]
    fn function_layer_matches_nocfree_shortcuts_by_key_identity() {
        for raw in 0..KEY_COUNT {
            match base_action(raw) {
                Action::Key(0x1e) => assert_eq!(function_action(raw), Action::ProfileShortcut(0)),
                Action::Key(0x1f) => assert_eq!(function_action(raw), Action::ProfileShortcut(1)),
                Action::Key(0x20) => assert_eq!(function_action(raw), Action::ProfileShortcut(2)),
                Action::Key(0x22) => assert_eq!(function_action(raw), Action::BootLeft),
                Action::Key(0x27) | Action::Key(0x4c) => {
                    assert_eq!(function_action(raw), Action::BootRight)
                }
                Action::Key(0x2b) => assert_eq!(function_action(raw), Action::BacklightToggle),
                Action::Key(0x0c) => assert_eq!(function_action(raw), Action::BatteryStatus),
                _ => {}
            }
        }
        assert_eq!(
            function_action_for(Action::Key(0x3e)),
            Action::BacklightDown
        );
        assert_eq!(function_action_for(Action::Key(0x3f)), Action::BacklightUp);
    }

    #[cfg(feature = "layout-ansi")]
    #[test]
    fn ansi_constants_and_known_raw_positions_are_unchanged() {
        assert_eq!((LAYOUT_ID, LAYOUT_NAME), (1, "ANSI"));
        assert_eq!((LEFT_KEY_COUNT, RIGHT_KEY_COUNT, KEY_COUNT), (37, 47, 84));
        assert_eq!(raw_to_visual(0), 0);
        assert_eq!(raw_to_visual(37), 7);
        assert_eq!(function_action(68), Action::BootRight);
        assert_eq!(function_action(8), Action::ProfileShortcut(0));
        assert_eq!(function_action(9), Action::ProfileShortcut(1));
        assert_eq!(function_action(10), Action::ProfileShortcut(2));
        assert_eq!(function_action(12), Action::BootLeft);
        assert_eq!(function_action(48), Action::BootRight);
        assert_eq!(function_action(14), Action::BacklightToggle);
        assert_eq!(function_action(55), Action::BatteryStatus);
    }

    #[cfg(feature = "layout-iso")]
    #[test]
    fn iso_matches_the_official_extra_left_key_and_non_us_usages() {
        assert_eq!((LAYOUT_ID, LAYOUT_NAME), (2, "ISO"));
        assert_eq!((LEFT_KEY_COUNT, RIGHT_KEY_COUNT, KEY_COUNT), (38, 47, 85));
        assert_eq!(LEFT_ROW_COUNTS, [7, 7, 6, 6, 7, 5]);
        assert!(
            (0..LEFT_KEY_COUNT).any(|raw| base_action(raw) == Action::Key(0x64)),
            "ISO must expose Keyboard Non-US Backslash"
        );
        assert!(
            (LEFT_KEY_COUNT..KEY_COUNT).any(|raw| base_action(raw) == Action::Key(0x32)),
            "ISO must expose Keyboard Non-US Hash"
        );
    }

    #[cfg(feature = "layout-jis")]
    #[test]
    fn jis_matches_the_hardware_tested_custom_zmk_mapping() {
        assert_eq!((LAYOUT_ID, LAYOUT_NAME), (4, "JIS"));
        assert_eq!((LEFT_KEY_COUNT, RIGHT_KEY_COUNT, KEY_COUNT), (37, 48, 85));
        assert_eq!(RIGHT_ROW_COUNTS, [8, 8, 8, 8, 8, 8]);
        for usage in [0x87, 0x89, 0x8a] {
            assert!(
                (0..KEY_COUNT).any(|raw| base_action(raw) == Action::Key(usage)),
                "JIS usage {usage:#04x} must be present"
            );
        }
    }

    #[cfg(feature = "layout-kr")]
    #[test]
    fn kr_duplicates_the_verified_boundary_keys() {
        // 이 테스트가 검증하는 시나리오: KR 경계 중복키 수와 HID 사용값이 실물 검증 결과를 유지한다.
        // Given / When: KR 배열의 키 수와 추가 포트 키 수를 조회한다.
        assert_eq!((LAYOUT_ID, LAYOUT_NAME), (3, "KR"));
        assert_eq!((LEFT_KEY_COUNT, RIGHT_KEY_COUNT, KEY_COUNT), (39, 50, 89));
        assert_eq!((EXTRA_LEFT_KEYS, EXTRA_RIGHT_KEYS), (0, 4));

        // Then: 다섯 경계 키는 양쪽에 정확히 하나씩 존재한다.
        for usage in [0x3f, 0x23, 0x1c, 0x0b, 0x05] {
            assert_eq!(
                (0..KEY_COUNT)
                    .filter(|raw| base_action(*raw) == Action::Key(usage))
                    .count(),
                2,
                "KR boundary usage {usage:#04x} must appear on both halves"
            );
        }
    }

    #[cfg(feature = "layout-kr")]
    #[test]
    fn kr_right_uses_the_verified_physical_row_order() {
        // 이 테스트가 검증하는 시나리오: KR 오른쪽 50키가 실물에서 누른 여섯 행의 raw 순서를 그대로 사용한다.
        // Given / When: 각 행의 오른쪽 visual 구간을 선택한다.
        let right_rows: [&[usize]; 6] = [
            &VISUAL_TO_RAW[7..17],
            &VISUAL_TO_RAW[24..33],
            &VISUAL_TO_RAW[40..49],
            &VISUAL_TO_RAW[56..63],
            &VISUAL_TO_RAW[69..77],
            &VISUAL_TO_RAW[82..89],
        ];

        // Then: 표준 포트와 0x21 P0.0~P0.3 키가 실제 왼쪽→오른쪽 순서와 일치한다.
        assert_eq!(RIGHT_ROW_COUNTS, [8, 8, 8, 7, 8, 7]);
        assert_eq!(RIGHT_FN_RAW, 80);
        assert_eq!(right_rows[0], [39, 40, 41, 42, 43, 44, 45, 46, 85, 86]);
        assert_eq!(right_rows[1], [47, 48, 49, 50, 51, 52, 53, 54, 87]);
        assert_eq!(right_rows[2], [55, 56, 57, 58, 59, 60, 61, 62, 88]);
        assert_eq!(right_rows[3], [63, 64, 65, 66, 67, 68, 69]);
        assert_eq!(right_rows[4], [70, 71, 72, 73, 74, 75, 76, 77]);
        assert_eq!(right_rows[5], [78, 79, 80, 81, 82, 83, 84]);
        assert_eq!(base_action(39), Action::Key(0x3f));
        assert_eq!(base_action(47), Action::Key(0x23));
        assert_eq!(base_action(70), Action::Key(0x05));
        assert_eq!(base_action(80), Action::Fn);
    }

    #[cfg(feature = "layout-kr")]
    #[test]
    fn kr_left_uses_the_verified_six_port_mapping() {
        // 이 테스트가 검증하는 시나리오: KR 왼쪽 39키가 0x21 추가 포트 없이 여섯 표준 포트에 배치된다.
        // Given: KR 배열을 선택한다.
        assert_eq!((LAYOUT_ID, LAYOUT_NAME), (3, "KR"));

        // When / Then: 실제 행 수, Fn 위치, Y/H raw 위치가 실물 포트 순서와 일치한다.
        assert_eq!(LEFT_ROW_COUNTS, [7, 7, 7, 7, 6, 5]);
        assert_eq!(EXTRA_LEFT_KEYS, 0);
        assert_eq!(LEFT_FN_RAW, 34);
        assert_eq!(base_action(20), Action::Key(0x1c));
        assert_eq!(base_action(27), Action::Key(0x0b));
    }
}
