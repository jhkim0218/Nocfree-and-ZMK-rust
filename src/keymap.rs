pub const KEY_COUNT: usize = 84;
pub const LEFT_KEY_COUNT: usize = 37;
pub const RIGHT_KEY_COUNT: usize = 47;
pub const LEFT_FN_RAW: usize = 32;
pub const RIGHT_FN_RAW: usize = 79;

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

macro_rules! k {
    ($usage:expr) => {
        Action::Key($usage)
    };
}

#[rustfmt::skip]
const VISUAL_BASE: [Action; KEY_COUNT] = [
    k!(0x29), k!(0x3a), k!(0x3b), k!(0x3c), k!(0x3d), k!(0x3e), k!(0x3f),
    k!(0x40), k!(0x41), k!(0x42), k!(0x43), k!(0x44), k!(0x45), k!(0x46), k!(0x4a),

    k!(0x35), k!(0x1e), k!(0x1f), k!(0x20), k!(0x21), k!(0x22), k!(0x23),
    k!(0x24), k!(0x25), k!(0x26), k!(0x27), k!(0x2d), k!(0x2e), k!(0x2a), k!(0x4b),

    k!(0x2b), k!(0x14), k!(0x1a), k!(0x08), k!(0x15), k!(0x17),
    k!(0x1c), k!(0x18), k!(0x0c), k!(0x12), k!(0x13), k!(0x2f), k!(0x30), k!(0x31),

    k!(0x39), k!(0x04), k!(0x16), k!(0x07), k!(0x09), k!(0x0a),
    k!(0x0b), k!(0x0d), k!(0x0e), k!(0x0f), k!(0x33), k!(0x34), k!(0x28), k!(0x4c),

    k!(0xe1), k!(0x1d), k!(0x1b), k!(0x06), k!(0x19), k!(0x05),
    k!(0x11), k!(0x10), k!(0x36), k!(0x37), k!(0x38), k!(0xe5), k!(0x52), k!(0x4e),

    Action::Fn, k!(0xe0), k!(0xe2), k!(0xe3), k!(0x2c),
    k!(0x2c), k!(0xe7), Action::Fn, k!(0xe6), k!(0x50), k!(0x51), k!(0x4f),
];

pub const fn raw_to_visual(raw: usize) -> usize {
    match raw {
        0..=6 => raw,
        7..=13 => raw + 8,
        14..=19 => raw + 16,
        20..=25 => raw + 24,
        26..=31 => raw + 32,
        32..=36 => raw + 40,
        37..=44 => raw - 30,
        45..=52 => raw - 23,
        53..=60 => raw - 17,
        61..=68 => raw - 11,
        69..=76 => raw - 5,
        77..=83 => raw,
        _ => panic!("raw key index outside the ANSI map"),
    }
}

pub const fn base_action(raw: usize) -> Action {
    VISUAL_BASE[raw_to_visual(raw)]
}

pub const fn function_action(raw: usize) -> Action {
    match raw_to_visual(raw) {
        0 => Action::ResetLeft,
        1 => Action::Consumer(0x006f),
        2 => Action::Consumer(0x0070),
        3 => Action::SystemF3,
        4 => Action::SystemF4,
        5 => Action::BacklightDown,
        6 => Action::BacklightUp,
        7 => Action::Consumer(0x00b6),
        8 => Action::Consumer(0x00cd),
        9 => Action::Consumer(0x00b5),
        10 => Action::Consumer(0x00e2),
        11 => Action::Consumer(0x00ea),
        12 => Action::Consumer(0x00e9),
        16 => Action::ProfileShortcut(0),
        17 => Action::ProfileShortcut(1),
        18 => Action::ProfileShortcut(2),
        20 => Action::BootLeft,
        25 => Action::BootRight,
        30 => Action::BacklightToggle,
        37 => Action::OutputUsb,
        38 => Action::BatteryStatus,
        64 => Action::SystemShortcut {
            system: 1,
            key: 0x11,
        },
        65 => Action::SystemShortcut {
            system: 0,
            key: 0x10,
        },
        57 => Action::BootRight,
        63 => Action::OutputBle,
        _ => Action::Transparent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transform_is_a_permutation() {
        let mut seen = [false; KEY_COUNT];
        for raw in 0..KEY_COUNT {
            let visual = raw_to_visual(raw);
            assert!(!seen[visual], "duplicate visual index {visual}");
            seen[visual] = true;
        }
        assert!(seen.into_iter().all(|value| value));
    }

    #[test]
    fn halves_and_function_keys_are_not_swapped() {
        assert_eq!(raw_to_visual(0), 0);
        assert_eq!(raw_to_visual(37), 7);
        assert_eq!(base_action(LEFT_FN_RAW), Action::Fn);
        assert_eq!(base_action(RIGHT_FN_RAW), Action::Fn);
        assert_eq!(function_action(0), Action::ResetLeft);
        assert_eq!(function_action(68), Action::BootRight);
    }

    #[test]
    fn function_layer_matches_nocfree_shortcuts() {
        assert_eq!(function_action(8), Action::ProfileShortcut(0));
        assert_eq!(function_action(9), Action::ProfileShortcut(1));
        assert_eq!(function_action(10), Action::ProfileShortcut(2));
        assert_eq!(function_action(12), Action::BootLeft);
        assert_eq!(function_action(48), Action::BootRight);
        assert_eq!(function_action(14), Action::BacklightToggle);
        assert_eq!(function_action(55), Action::BatteryStatus);
        assert_eq!(
            function_action(69),
            Action::SystemShortcut {
                system: 1,
                key: 0x11
            }
        );
        assert_eq!(
            function_action(70),
            Action::SystemShortcut {
                system: 0,
                key: 0x10
            }
        );
        assert_eq!(function_action(54), Action::OutputUsb);
        assert_eq!(function_action(31), Action::OutputBle);
        assert_eq!(function_action(1), Action::Consumer(0x006f));
        assert_eq!(function_action(2), Action::Consumer(0x0070));
        assert_eq!(function_action(3), Action::SystemF3);
        assert_eq!(function_action(4), Action::SystemF4);
        assert_eq!(function_action(5), Action::BacklightDown);
        assert_eq!(function_action(6), Action::BacklightUp);
        assert_eq!(function_action(37), Action::Consumer(0x00b6));
        assert_eq!(function_action(38), Action::Consumer(0x00cd));
        assert_eq!(function_action(39), Action::Consumer(0x00b5));
        assert_eq!(function_action(40), Action::Consumer(0x00e2));
        assert_eq!(function_action(41), Action::Consumer(0x00ea));
        assert_eq!(function_action(42), Action::Consumer(0x00e9));
    }
}
