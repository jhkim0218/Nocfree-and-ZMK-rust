pub use crate::keymap::raw_from_matrix;
use crate::keymap::{
    Action, KEY_COUNT, LAYOUT_ID, MATRIX_COLS, MATRIX_ROWS, base_action, function_action,
    matrix_from_raw,
};

pub const LINK_LAYERS: usize = 8;
pub const LINK_ROWS: usize = MATRIX_ROWS;
pub const LINK_COLS: usize = MATRIX_COLS;
pub const LINK_BINDING_BYTES: usize = 3;
pub const LINK_KEYMAP_BYTES: usize = LINK_LAYERS * KEY_COUNT * LINK_BINDING_BYTES;
pub const HOTKEY_SLOTS: usize = 16;
const HOTKEY_BYTES: usize = 8;
pub const LINK_KEYMAP_RECORD_BYTES: usize = 12 + LINK_KEYMAP_BYTES + HOTKEY_SLOTS * HOTKEY_BYTES;
pub const LINK_KEYMAP_PAGE: u8 = 7;

const MAGIC: [u8; 4] = *b"NFK1";
const VERSION: u8 = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LinkBinding {
    pub kind: u8,
    pub value: u16,
}

impl LinkBinding {
    pub const TRANSPARENT: Self = Self { kind: 0, value: 0 };

    pub const fn new(kind: u8, value: u16) -> Self {
        Self { kind, value }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HotkeySlot {
    pub empty: bool,
    pub row: u8,
    pub col: u8,
    pub layer: u8,
    pub modifiers: u8,
    pub key: u16,
}

impl HotkeySlot {
    pub const EMPTY: Self = Self {
        empty: true,
        row: u8::MAX,
        col: u8::MAX,
        layer: u8::MAX,
        modifiers: 0,
        key: 0,
    };

    pub const fn action(self) -> Action {
        if self.empty || self.key > u8::MAX as u16 {
            Action::NoAction
        } else {
            Action::Chord {
                modifiers: self.modifiers,
                key: self.key as u8,
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LinkKeymap {
    bindings: [LinkBinding; LINK_LAYERS * KEY_COUNT],
    hotkeys: [HotkeySlot; HOTKEY_SLOTS],
    system: u8,
}

impl Default for LinkKeymap {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkKeymap {
    pub const fn new() -> Self {
        let mut map = Self {
            bindings: [LinkBinding::TRANSPARENT; LINK_LAYERS * KEY_COUNT],
            hotkeys: [HotkeySlot::EMPTY; HOTKEY_SLOTS],
            system: 1,
        };
        let mut raw = 0;
        while raw < KEY_COUNT {
            map.bindings[raw] = binding_from_action(base_action(raw));
            map.bindings[KEY_COUNT + raw] = binding_from_action(function_action(raw));
            raw += 1;
        }
        map
    }

    pub fn binding(&self, layer: usize, raw: usize) -> Option<LinkBinding> {
        (layer < LINK_LAYERS && raw < KEY_COUNT).then_some(self.bindings[layer * KEY_COUNT + raw])
    }

    pub fn action(&self, layer: usize, raw: usize) -> Action {
        let Some(binding) = self.binding(layer, raw) else {
            return Action::NoAction;
        };
        if binding.kind == 3 && (64..64 + HOTKEY_SLOTS as u16).contains(&binding.value) {
            return self.hotkeys[(binding.value - 64) as usize].action();
        }
        if let Some((row, col)) = matrix_from_raw(raw)
            && let Some(slot) = self.hotkeys.iter().find(|slot| {
                !slot.empty
                    && slot.layer as usize == layer
                    && slot.row as usize == row
                    && slot.col as usize == col
            })
        {
            return slot.action();
        }
        action_from_binding(binding)
    }

    pub fn matrix_binding(&self, layer: usize, row: usize, col: usize) -> Option<LinkBinding> {
        self.binding(layer, raw_from_matrix(row, col)?)
    }

    pub fn set_matrix_binding(
        &mut self,
        layer: usize,
        row: usize,
        col: usize,
        binding: LinkBinding,
    ) -> bool {
        let Some(raw) = raw_from_matrix(row, col) else {
            return false;
        };
        if layer >= LINK_LAYERS {
            return false;
        }
        self.bindings[layer * KEY_COUNT + raw] = binding;
        true
    }

    pub fn reset_layer(&mut self, layer: usize) -> bool {
        if layer >= LINK_LAYERS {
            return false;
        }
        let defaults = Self::default();
        let start = layer * KEY_COUNT;
        self.bindings[start..start + KEY_COUNT]
            .copy_from_slice(&defaults.bindings[start..start + KEY_COUNT]);
        true
    }

    pub fn reset_all(&mut self) {
        self.bindings = Self::default().bindings;
    }

    pub fn hotkey(&self, slot: usize) -> Option<HotkeySlot> {
        self.hotkeys.get(slot).copied()
    }

    pub fn set_hotkey(&mut self, slot: usize, hotkey: HotkeySlot) -> bool {
        let Some(current) = self.hotkeys.get_mut(slot) else {
            return false;
        };
        *current = hotkey;
        true
    }

    pub fn clear_hotkey(&mut self, slot: usize) -> bool {
        self.set_hotkey(slot, HotkeySlot::EMPTY)
    }

    pub fn system(&self) -> u8 {
        self.system
    }

    pub fn set_system(&mut self, system: u8) -> bool {
        if system > 1 {
            return false;
        }
        self.system = system;
        true
    }

    pub fn encode(&self) -> [u8; LINK_KEYMAP_RECORD_BYTES] {
        let mut bytes = [0xff; LINK_KEYMAP_RECORD_BYTES];
        bytes[..4].copy_from_slice(&MAGIC);
        bytes[4] = VERSION;
        bytes[5] = self.system;
        bytes[6] = LAYOUT_ID;
        bytes[7] = KEY_COUNT as u8;
        let mut offset = 8;
        for binding in self.bindings {
            bytes[offset] = binding.kind;
            bytes[offset + 1..offset + 3].copy_from_slice(&binding.value.to_le_bytes());
            offset += LINK_BINDING_BYTES;
        }
        for slot in self.hotkeys {
            bytes[offset] = u8::from(!slot.empty);
            bytes[offset + 1] = slot.row;
            bytes[offset + 2] = slot.col;
            bytes[offset + 3] = slot.layer;
            bytes[offset + 4] = slot.modifiers;
            bytes[offset + 5..offset + 7].copy_from_slice(&slot.key.to_le_bytes());
            bytes[offset + 7] = 0;
            offset += HOTKEY_BYTES;
        }
        let crc = crc32(&bytes[..LINK_KEYMAP_RECORD_BYTES - 4]);
        bytes[LINK_KEYMAP_RECORD_BYTES - 4..].copy_from_slice(&crc.to_le_bytes());
        bytes
    }

    pub fn decode(bytes: &[u8; LINK_KEYMAP_RECORD_BYTES]) -> Option<Self> {
        if bytes[..4] != MAGIC
            || bytes[4] != VERSION
            || bytes[5] > 1
            || bytes[6] != LAYOUT_ID
            || bytes[7] as usize != KEY_COUNT
        {
            return None;
        }
        let crc_offset = LINK_KEYMAP_RECORD_BYTES - 4;
        let stored = u32::from_le_bytes(bytes[crc_offset..].try_into().ok()?);
        if stored != crc32(&bytes[..crc_offset]) {
            return None;
        }
        let mut map = Self {
            bindings: [LinkBinding::TRANSPARENT; LINK_LAYERS * KEY_COUNT],
            hotkeys: [HotkeySlot::EMPTY; HOTKEY_SLOTS],
            system: bytes[5],
        };
        let mut offset = 8;
        for binding in &mut map.bindings {
            *binding = LinkBinding {
                kind: bytes[offset],
                value: u16::from_le_bytes(bytes[offset + 1..offset + 3].try_into().ok()?),
            };
            offset += LINK_BINDING_BYTES;
        }
        for slot in &mut map.hotkeys {
            *slot = HotkeySlot {
                empty: bytes[offset] == 0,
                row: bytes[offset + 1],
                col: bytes[offset + 2],
                layer: bytes[offset + 3],
                modifiers: bytes[offset + 4],
                key: u16::from_le_bytes(bytes[offset + 5..offset + 7].try_into().ok()?),
            };
            offset += HOTKEY_BYTES;
        }
        Some(map)
    }
}

const fn binding_from_action(action: Action) -> LinkBinding {
    match action {
        Action::Transparent => LinkBinding::TRANSPARENT,
        Action::NoAction => LinkBinding::new(0, 1),
        Action::Fn => LinkBinding::new(3, 1),
        Action::Key(usage @ 0xe0..=0xe7) => LinkBinding::new(1, usage as u16),
        Action::Key(usage) => LinkBinding::new(0, usage as u16),
        Action::Chord { .. } | Action::LayerMomentary(_) | Action::LayerToggle(_) => {
            LinkBinding::TRANSPARENT
        }
        Action::Consumer(0x00e2) => LinkBinding::new(2, 168),
        Action::Consumer(0x00e9) => LinkBinding::new(2, 169),
        Action::Consumer(0x00ea) => LinkBinding::new(2, 170),
        Action::Consumer(usage) => LinkBinding::new(2, usage),
        Action::ResetLeft => LinkBinding::new(0x80, 1),
        Action::BootRight => LinkBinding::new(0x80, 2),
        Action::BootLeft => LinkBinding::new(0x80, 3),
        Action::ProfileSelect(profile) => LinkBinding::new(0x80, 0x10 + profile as u16),
        Action::ProfileShortcut(profile) => LinkBinding::new(0x80, 0x40 + profile as u16),
        Action::ProfileClear => LinkBinding::new(0x80, 0x20),
        Action::OutputUsb => LinkBinding::new(0x80, 0x30),
        Action::OutputBle => LinkBinding::new(0x80, 0x31),
        Action::BacklightToggle => LinkBinding::new(0x80, 0x50),
        Action::BacklightDown => LinkBinding::new(0x80, 0x51),
        Action::BacklightUp => LinkBinding::new(0x80, 0x52),
        Action::BatteryStatus => LinkBinding::new(0x80, 0x53),
        Action::SystemF3 => LinkBinding::new(0x80, 0x54),
        Action::SystemF4 => LinkBinding::new(0x80, 0x55),
        Action::SystemShortcut {
            system: 0,
            key: 0x10,
        } => LinkBinding::new(0x80, 0x56),
        Action::SystemShortcut {
            system: 1,
            key: 0x11,
        } => LinkBinding::new(0x80, 0x57),
        Action::SystemShortcut { .. } => LinkBinding::new(0x00, 0),
    }
}

pub const fn action_from_binding(binding: LinkBinding) -> Action {
    match binding.kind {
        0 if binding.value == 0 => Action::Transparent,
        0 if binding.value == 1 => Action::NoAction,
        0 if binding.value <= 0xff => Action::Key(binding.value as u8),
        1 if binding.value >= 0xe0 && binding.value <= 0xe7 => Action::Key(binding.value as u8),
        2 => Action::Consumer(match binding.value {
            127 => 0x00e2,
            128 => 0x00e9,
            129 => 0x00ea,
            165 => 0x0030,
            166 => 0x0032,
            167 => 0x0033,
            168 => 0x00e2,
            169 => 0x00e9,
            170 => 0x00ea,
            171 => 0x00b5,
            172 => 0x00b6,
            173 => 0x00b7,
            174 => 0x00cd,
            176 => 0x00b8,
            180 => 0x0221,
            181 => 0x0223,
            182 => 0x0224,
            183 => 0x0225,
            184 => 0x0226,
            185 => 0x0227,
            186 => 0x022a,
            187 => 0x00b3,
            188 => 0x00b4,
            189 => 0x006f,
            190 => 0x0070,
            value => value,
        }),
        3 => match binding.value {
            1 | 97 => Action::LayerMomentary(1),
            2..=4 => Action::ProfileSelect((binding.value - 2) as u8),
            5 => Action::Chord {
                modifiers: 1 << 3,
                key: 0x07,
            },
            8 => Action::Chord {
                modifiers: 1 << 0,
                key: 0x52,
            },
            9 => Action::Chord {
                modifiers: 1 << 3,
                key: 0x2c,
            },
            17 => Action::Chord {
                modifiers: 1 << 3,
                key: 0x0f,
            },
            19 => Action::ResetLeft,
            96 | 98 => Action::LayerToggle(1),
            99 | 101 | 103 | 105 | 107 | 109 => {
                Action::LayerMomentary(((binding.value - 95) / 2) as u8)
            }
            100 | 102 | 104 | 106 | 108 | 110 => {
                Action::LayerToggle(((binding.value - 96) / 2) as u8)
            }
            value if value & 0x3000 == 0x2000 => Action::LayerMomentary(((value >> 8) & 7) as u8),
            value if value & 0x3000 == 0x3000 => {
                let modifier_slot = ((value >> 8) & 0x0f) as u8;
                let modifier = if modifier_slot < 8 {
                    1 << modifier_slot
                } else {
                    0
                };
                Action::Chord {
                    modifiers: modifier,
                    key: value as u8,
                }
            }
            _ => Action::NoAction,
        },
        0x80 => match binding.value {
            1 => Action::ResetLeft,
            2 => Action::BootRight,
            3 => Action::BootLeft,
            0x10..=0x12 => Action::ProfileSelect((binding.value - 0x10) as u8),
            0x20 => Action::ProfileClear,
            0x30 => Action::OutputUsb,
            0x31 => Action::OutputBle,
            0x40..=0x42 => Action::ProfileShortcut((binding.value - 0x40) as u8),
            0x50 => Action::BacklightToggle,
            0x51 => Action::BacklightDown,
            0x52 => Action::BacklightUp,
            0x53 => Action::BatteryStatus,
            0x54 => Action::SystemF3,
            0x55 => Action::SystemF4,
            0x56 => Action::SystemShortcut {
                system: 0,
                key: 0x10,
            },
            0x57 => Action::SystemShortcut {
                system: 1,
                key: 0x11,
            },
            _ => Action::NoAction,
        },
        _ => Action::NoAction,
    }
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

    #[test]
    fn matrix_coordinates_cover_each_physical_key_once() {
        let mut seen = [false; KEY_COUNT];
        for row in 0..LINK_ROWS {
            for col in 0..LINK_COLS {
                if let Some(raw) = raw_from_matrix(row, col) {
                    assert!(!seen[raw]);
                    seen[raw] = true;
                    assert_eq!(matrix_from_raw(raw), Some((row, col)));
                }
            }
        }
        assert!(seen.into_iter().all(|value| value));
    }

    #[test]
    fn default_map_preserves_the_existing_zmk_layers() {
        let map = LinkKeymap::default();
        assert_eq!(map.matrix_binding(0, 0, 0), Some(LinkBinding::new(0, 0x29)));
        assert_eq!(map.matrix_binding(0, 2, 6), Some(LinkBinding::new(0, 0x1c)));
        let fn_raw = (0..KEY_COUNT)
            .find(|raw| base_action(*raw) == Action::Fn)
            .unwrap();
        let (fn_row, fn_col) = matrix_from_raw(fn_raw).unwrap();
        assert_eq!(
            map.matrix_binding(0, fn_row, fn_col),
            Some(LinkBinding::new(3, 1))
        );
        let mute_raw = (0..KEY_COUNT)
            .find(|raw| function_action(*raw) == Action::Consumer(0x00e2))
            .unwrap();
        let (row, col) = matrix_from_raw(mute_raw).unwrap();
        assert_eq!(
            map.matrix_binding(1, row, col),
            Some(LinkBinding::new(2, 168))
        );
    }

    #[test]
    fn keymap_record_round_trips_and_rejects_corruption() {
        let mut map = LinkKeymap::default();
        assert!(map.set_matrix_binding(7, 4, 10, LinkBinding::new(2, 174)));
        assert!(map.set_hotkey(
            3,
            HotkeySlot {
                empty: false,
                row: u8::MAX,
                col: u8::MAX,
                layer: u8::MAX,
                modifiers: 1 << 0,
                key: 0x06,
            }
        ));
        assert!(map.set_system(0));
        let mut encoded = map.encode();
        assert_eq!(encoded[4], VERSION);
        assert_eq!(encoded[6], LAYOUT_ID);
        assert_eq!(encoded[7] as usize, KEY_COUNT);
        assert_eq!(LinkKeymap::decode(&encoded), Some(map));
        encoded[4] = VERSION - 1;
        assert_eq!(LinkKeymap::decode(&encoded), None);
        encoded[4] = VERSION;
        encoded[100] ^= 1;
        assert_eq!(LinkKeymap::decode(&encoded), None);
    }

    #[test]
    fn reset_restores_stock_bindings() {
        let mut map = LinkKeymap::default();
        assert!(map.set_matrix_binding(1, 3, 7, LinkBinding::new(0, 0x04)));
        assert!(map.reset_layer(1));
        assert_eq!(map, LinkKeymap::default());
    }

    #[test]
    fn nocfree_bindings_convert_to_hid_and_layers() {
        assert_eq!(
            action_from_binding(LinkBinding::new(2, 174)),
            Action::Consumer(0x00cd)
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 99)),
            Action::LayerMomentary(2)
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 104)),
            Action::LayerToggle(4)
        );
    }

    #[test]
    fn hotkeys_execute_by_map_value_or_bound_coordinate() {
        let mut map = LinkKeymap::default();
        let hotkey = HotkeySlot {
            empty: false,
            row: 3,
            col: 7,
            layer: 2,
            modifiers: (1 << 0) | (1 << 2),
            key: 0x4c,
        };
        assert!(map.set_hotkey(0, hotkey));
        assert!(map.set_matrix_binding(0, 0, 0, LinkBinding::new(3, 64)));
        let expected = Action::Chord {
            modifiers: hotkey.modifiers,
            key: hotkey.key as u8,
        };
        assert_eq!(map.action(0, 0), expected);
        assert_eq!(map.action(2, raw_from_matrix(3, 7).unwrap()), expected);
    }
}
