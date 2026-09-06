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
// KR은 양쪽 raw 순서 교정 때마다 이전 저장 키맵을 무효화한다.
#[cfg(feature = "layout-kr")]
const VERSION: u8 = 8;
#[cfg(not(feature = "layout-kr"))]
const VERSION: u8 = 6;
#[cfg(feature = "layout-kr")]
const PREVIOUS_VERSION: u8 = 7;
#[cfg(not(feature = "layout-kr"))]
const PREVIOUS_VERSION: u8 = 5;

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
            if map.bindings[KEY_COUNT + raw].kind == 0 && map.bindings[KEY_COUNT + raw].value == 0 {
                map.bindings[KEY_COUNT + raw] = map.bindings[raw];
            }
            raw += 1;
        }
        let mut layer = 2;
        while layer < LINK_LAYERS {
            raw = 0;
            while raw < KEY_COUNT {
                map.bindings[layer * KEY_COUNT + raw] = map.bindings[raw];
                raw += 1;
            }
            layer += 1;
        }
        map
    }

    pub fn binding(&self, layer: usize, raw: usize) -> Option<LinkBinding> {
        if layer >= LINK_LAYERS || raw >= KEY_COUNT {
            return None;
        }
        Some(self.bindings[layer * KEY_COUNT + raw])
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
        let raw = raw_from_matrix(row, col)?;
        let binding = self.binding(layer, raw)?;
        if let Some(slot) = self.hotkeys.iter().position(|slot| {
            slot.is_coordinate_bound()
                && slot.layer as usize == layer
                && slot.row as usize == row
                && slot.col as usize == col
        }) {
            return Some(LinkBinding::new(3, 64 + slot as u16));
        }
        Some(binding)
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
        self.clear_coordinate_hotkey(layer, row, col);
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
        for hotkey in &mut self.hotkeys {
            if hotkey.is_coordinate_bound() && hotkey.layer as usize == layer {
                *hotkey = HotkeySlot::EMPTY;
            }
        }
        true
    }

    pub fn reset_all(&mut self) {
        self.bindings = Self::default().bindings;
        for hotkey in &mut self.hotkeys {
            if hotkey.is_coordinate_bound() {
                *hotkey = HotkeySlot::EMPTY;
            }
        }
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
        Self::decode_with_migration(bytes).map(|(map, _)| map)
    }

    pub fn decode_with_migration(bytes: &[u8; LINK_KEYMAP_RECORD_BYTES]) -> Option<(Self, bool)> {
        if bytes[..4] != MAGIC
            || (bytes[4] != VERSION && bytes[4] != PREVIOUS_VERSION)
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
        let migrated = bytes[4] == PREVIOUS_VERSION;
        if migrated {
            map.copy_base_bindings_to_empty_function_keys();
            for layer in 2..LINK_LAYERS {
                map.copy_base_layer_to(layer);
            }
        }
        Some((map, migrated))
    }

    fn copy_base_layer_to(&mut self, layer: usize) {
        let start = layer * KEY_COUNT;
        for raw in 0..KEY_COUNT {
            self.bindings[start + raw] = self.bindings[raw];
        }
        for hotkey in &mut self.hotkeys {
            if hotkey.is_coordinate_bound() && hotkey.layer as usize == layer {
                *hotkey = HotkeySlot::EMPTY;
            }
        }
    }

    fn copy_base_bindings_to_empty_function_keys(&mut self) {
        for raw in 0..KEY_COUNT {
            let function_index = KEY_COUNT + raw;
            if self.bindings[function_index] == LinkBinding::TRANSPARENT {
                self.bindings[function_index] = self.bindings[raw];
            }
        }
    }

    fn clear_coordinate_hotkey(&mut self, layer: usize, row: usize, col: usize) {
        for hotkey in &mut self.hotkeys {
            if hotkey.is_coordinate_bound()
                && hotkey.layer as usize == layer
                && hotkey.row as usize == row
                && hotkey.col as usize == col
            {
                *hotkey = HotkeySlot::EMPTY;
            }
        }
    }
}

impl HotkeySlot {
    const fn is_coordinate_bound(self) -> bool {
        (self.layer as usize) < LINK_LAYERS
            && (self.row as usize) < LINK_ROWS
            && (self.col as usize) < LINK_COLS
            && raw_from_matrix(self.row as usize, self.col as usize).is_some()
    }
}

pub const fn binding_for_link(binding: LinkBinding) -> LinkBinding {
    match binding {
        LinkBinding {
            kind: 0x80,
            value: 0x40..=0x42,
        } => LinkBinding::new(3, binding.value - 0x3e),
        LinkBinding {
            kind: 0x80,
            value: 0x50,
        } => LinkBinding::new(3, 34),
        LinkBinding {
            kind: 0x80,
            value: 0x51,
        } => LinkBinding::new(3, 33),
        LinkBinding {
            kind: 0x80,
            value: 0x52,
        } => LinkBinding::new(3, 32),
        LinkBinding {
            kind: 0x80,
            value: 2,
        } => LinkBinding::new(3, 20),
        LinkBinding {
            kind: 0x80,
            value: 3,
        } => LinkBinding::new(3, 19),
        LinkBinding {
            kind: 2,
            value: 127,
        } => LinkBinding::new(2, 168),
        LinkBinding {
            kind: 2,
            value: 128,
        } => LinkBinding::new(2, 169),
        LinkBinding {
            kind: 2,
            value: 129,
        } => LinkBinding::new(2, 170),
        _ => binding,
    }
}

pub const fn binding_is_valid(binding: LinkBinding) -> bool {
    match binding.kind {
        0 => {
            binding.value <= 1
                || (binding.value >= 0x04
                    && binding.value
                        <= if cfg!(feature = "layout-jis") {
                            0x8a
                        } else {
                            0x73
                        })
        }
        1 => binding.value >= 0xe0 && binding.value <= 0xe7,
        2 => matches!(
            binding.value,
            127..=129 | 165..=176 | 180..=192
        ),
        3 => matches!(
            binding.value,
            1..=9 | 16..=17 | 19..=20 | 32..=34 | 64..=79 | 96..=110
        ),
        0x80 => matches!(
            binding.value,
            1..=3 | 0x10..=0x12 | 0x20 | 0x30..=0x31 | 0x40..=0x42 | 0x50..=0x57
        ),
        _ => false,
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
        Action::Consumer(0x0030) => LinkBinding::new(2, 165),
        Action::Consumer(0x0032) => LinkBinding::new(2, 166),
        Action::Consumer(0x0033) => LinkBinding::new(2, 167),
        Action::Consumer(0x00e2) => LinkBinding::new(2, 168),
        Action::Consumer(0x00e9) => LinkBinding::new(2, 169),
        Action::Consumer(0x00ea) => LinkBinding::new(2, 170),
        Action::Consumer(0x00b5) => LinkBinding::new(2, 171),
        Action::Consumer(0x00b6) => LinkBinding::new(2, 172),
        Action::Consumer(0x00b7) => LinkBinding::new(2, 173),
        Action::Consumer(0x00cd) => LinkBinding::new(2, 174),
        Action::Consumer(0x0183) => LinkBinding::new(2, 175),
        Action::Consumer(0x00b8) => LinkBinding::new(2, 176),
        Action::Consumer(0x0221) => LinkBinding::new(2, 180),
        Action::Consumer(0x0223) => LinkBinding::new(2, 181),
        Action::Consumer(0x0224) => LinkBinding::new(2, 182),
        Action::Consumer(0x0225) => LinkBinding::new(2, 183),
        Action::Consumer(0x0226) => LinkBinding::new(2, 184),
        Action::Consumer(0x0227) => LinkBinding::new(2, 185),
        Action::Consumer(0x022a) => LinkBinding::new(2, 186),
        Action::Consumer(0x00b3) => LinkBinding::new(2, 187),
        Action::Consumer(0x00b4) => LinkBinding::new(2, 188),
        Action::Consumer(0x006f) => LinkBinding::new(2, 189),
        Action::Consumer(0x0070) => LinkBinding::new(2, 190),
        Action::Consumer(0x019f) => LinkBinding::new(2, 191),
        Action::Consumer(0x00cf) => LinkBinding::new(2, 192),
        Action::Consumer(_) => LinkBinding::TRANSPARENT,
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
        Action::SystemSelect(1) => LinkBinding::new(3, 6),
        Action::SystemSelect(0) => LinkBinding::new(3, 7),
        Action::SystemSelect(_) => LinkBinding::TRANSPARENT,
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
        0 if binding.value >= 0x04
            && binding.value
                <= if cfg!(feature = "layout-jis") {
                    0x8a
                } else {
                    0x73
                } =>
        {
            Action::Key(binding.value as u8)
        }
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
            175 => 0x0183,
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
            191 => 0x019f,
            192 => 0x00cf,
            _ => return Action::NoAction,
        }),
        3 => match binding.value {
            1 | 97 => Action::LayerMomentary(1),
            2..=4 => Action::ProfileShortcut((binding.value - 2) as u8),
            5 => Action::Chord {
                modifiers: 1 << 3,
                key: 0x07,
            },
            6 => Action::SystemSelect(1),
            7 => Action::SystemSelect(0),
            8 => Action::Chord {
                modifiers: 1 << 0,
                key: 0x52,
            },
            9 => Action::Chord {
                modifiers: 1 << 3,
                key: 0x2c,
            },
            16 => Action::Consumer(0x00cf),
            17 => Action::Chord {
                modifiers: 1 << 3,
                key: 0x0f,
            },
            19 => Action::BootLeft,
            20 => Action::BootRight,
            32 => Action::BacklightUp,
            33 => Action::BacklightDown,
            34 => Action::BacklightToggle,
            96 => Action::LayerToggle(0),
            98 => Action::LayerToggle(1),
            99 | 101 | 103 | 105 | 107 | 109 => {
                Action::LayerMomentary(((binding.value - 95) / 2) as u8)
            }
            100 | 102 | 104 | 106 | 108 | 110 => {
                Action::LayerToggle(((binding.value - 96) / 2) as u8)
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
        assert_eq!(map.system(), 1, "new keymaps must start in Windows mode");
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
        for raw in 0..KEY_COUNT {
            let expected = binding_from_action(function_action(raw));
            assert_eq!(
                map.binding(1, raw),
                Some(if expected == LinkBinding::TRANSPARENT {
                    binding_from_action(base_action(raw))
                } else {
                    expected
                })
            );
        }
        for layer in 2..LINK_LAYERS {
            for raw in 0..KEY_COUNT {
                assert_eq!(map.binding(layer, raw), map.binding(0, raw));
            }
        }
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
        encoded[4] = VERSION + 1;
        assert_eq!(LinkKeymap::decode(&encoded), None);
        encoded[4] = VERSION;
        encoded[100] ^= 1;
        assert_eq!(LinkKeymap::decode(&encoded), None);
    }

    #[cfg(feature = "layout-kr")]
    #[test]
    fn kr_mapping_change_invalidates_the_old_persisted_keymap() {
        // 이 테스트가 검증하는 시나리오: KR 오른쪽 raw 순서 변경 후 version 5 저장 키맵이 새 기본 매핑을 덮어쓰지 않는다.
        // Given: 새 KR 기본 키맵을 인코딩한다.
        let encoded = LinkKeymap::default().encode();

        // When / Then: KR 레코드는 version 8을 사용해 기존 version 7과 구분된다.
        assert_eq!(VERSION, 8);
        assert_eq!(encoded[4], 8);
    }

    #[test]
    fn previous_keymap_version_preserves_first_two_layers_and_rebuilds_higher_layers() {
        let mut map = LinkKeymap::default();
        assert!(map.set_matrix_binding(0, 0, 0, LinkBinding::new(0, 0x04)));
        assert!(map.set_matrix_binding(1, 0, 0, LinkBinding::new(0, 0x05)));
        assert!(map.set_matrix_binding(0, 0, 1, LinkBinding::new(0, 0x06)));
        assert!(map.set_matrix_binding(1, 0, 1, LinkBinding::TRANSPARENT));
        assert!(map.set_matrix_binding(2, 0, 0, LinkBinding::new(2, 174)));
        let mut encoded = map.encode();
        encoded[4] = PREVIOUS_VERSION;
        let crc = crc32(&encoded[..LINK_KEYMAP_RECORD_BYTES - 4]);
        encoded[LINK_KEYMAP_RECORD_BYTES - 4..].copy_from_slice(&crc.to_le_bytes());

        let (migrated, needs_save) = LinkKeymap::decode_with_migration(&encoded).unwrap();
        assert!(needs_save);
        assert_eq!(migrated.binding(0, 0), Some(LinkBinding::new(0, 0x04)));
        assert_eq!(migrated.binding(1, 0), Some(LinkBinding::new(0, 0x05)));
        let raw = raw_from_matrix(0, 1).unwrap();
        assert_eq!(migrated.binding(1, raw), migrated.binding(0, raw));
        for layer in 2..LINK_LAYERS {
            assert_eq!(migrated.binding(layer, 0), migrated.binding(0, 0));
        }
        assert_eq!(migrated.encode()[4], VERSION);
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
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 96)),
            Action::LayerToggle(0)
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 97)),
            Action::LayerMomentary(1)
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 98)),
            Action::LayerToggle(1)
        );
    }

    #[test]
    fn link_function_layers_cover_the_eight_layers_shown_by_link() {
        for (value, action) in [
            (96, Action::LayerToggle(0)),
            (97, Action::LayerMomentary(1)),
            (98, Action::LayerToggle(1)),
            (99, Action::LayerMomentary(2)),
            (100, Action::LayerToggle(2)),
            (101, Action::LayerMomentary(3)),
            (102, Action::LayerToggle(3)),
            (103, Action::LayerMomentary(4)),
            (104, Action::LayerToggle(4)),
            (105, Action::LayerMomentary(5)),
            (106, Action::LayerToggle(5)),
            (107, Action::LayerMomentary(6)),
            (108, Action::LayerToggle(6)),
            (109, Action::LayerMomentary(7)),
            (110, Action::LayerToggle(7)),
        ] {
            let binding = LinkBinding::new(3, value);
            assert!(binding_is_valid(binding));
            assert_eq!(action_from_binding(binding), action);
        }
    }

    #[test]
    fn link_function_voice_and_system_entries_have_hid_actions() {
        for (binding, action) in [
            (LinkBinding::new(3, 16), Action::Consumer(0x00cf)),
            (LinkBinding::new(2, 175), Action::Consumer(0x0183)),
            (LinkBinding::new(2, 191), Action::Consumer(0x019f)),
            (LinkBinding::new(2, 192), Action::Consumer(0x00cf)),
        ] {
            assert!(binding_is_valid(binding));
            assert_eq!(action_from_binding(binding), action);
        }
    }

    #[test]
    fn canonical_link_bindings_preserve_ble_and_backlight_actions() {
        for (legacy, web, action) in [
            (
                LinkBinding::new(0x80, 0x40),
                LinkBinding::new(3, 2),
                Action::ProfileShortcut(0),
            ),
            (
                LinkBinding::new(0x80, 0x41),
                LinkBinding::new(3, 3),
                Action::ProfileShortcut(1),
            ),
            (
                LinkBinding::new(0x80, 0x42),
                LinkBinding::new(3, 4),
                Action::ProfileShortcut(2),
            ),
            (
                LinkBinding::new(0x80, 0x50),
                LinkBinding::new(3, 34),
                Action::BacklightToggle,
            ),
            (
                LinkBinding::new(0x80, 0x51),
                LinkBinding::new(3, 33),
                Action::BacklightDown,
            ),
            (
                LinkBinding::new(0x80, 0x52),
                LinkBinding::new(3, 32),
                Action::BacklightUp,
            ),
            (
                LinkBinding::new(0x80, 3),
                LinkBinding::new(3, 19),
                Action::BootLeft,
            ),
        ] {
            assert_eq!(binding_for_link(legacy), web);
            assert_eq!(action_from_binding(legacy), action);
            assert_eq!(action_from_binding(web), action);
            assert!(binding_is_valid(web));
        }
    }

    #[test]
    fn consumer_bindings_round_trip_default_function_actions() {
        for action in [
            Action::Consumer(0x006f),
            Action::Consumer(0x0070),
            Action::Consumer(0x00b5),
            Action::Consumer(0x00b6),
            Action::Consumer(0x00cd),
            Action::Consumer(0x00e2),
            Action::Consumer(0x00e9),
            Action::Consumer(0x00ea),
        ] {
            let binding = binding_from_action(action);
            assert!(binding_is_valid(binding));
            assert_eq!(action_from_binding(binding), action);
        }
    }

    #[test]
    fn web_system_and_left_dfu_codes_keep_their_distinct_meanings() {
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 6)),
            Action::SystemSelect(1)
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 7)),
            Action::SystemSelect(0)
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(3, 19)),
            Action::BootLeft
        );
        assert_eq!(
            action_from_binding(LinkBinding::new(0x80, 1)),
            Action::ResetLeft
        );
        assert_eq!(
            binding_from_action(Action::SystemSelect(1)),
            LinkBinding::new(3, 6)
        );
        assert_eq!(
            binding_from_action(Action::SystemSelect(0)),
            LinkBinding::new(3, 7)
        );
    }

    #[test]
    fn unsupported_bindings_are_rejected_and_legacy_unknowns_are_unchanged() {
        let bindings = [
            LinkBinding::new(0, 2),
            LinkBinding::new(2, 177),
            LinkBinding::new(2, 193),
            LinkBinding::new(3, 0x2001),
            LinkBinding::new(3, 0x3004),
            LinkBinding::new(0x80, 0x58),
        ];
        for value in 205..=223 {
            assert!(!binding_is_valid(LinkBinding::new(0, value)));
        }
        for value in 80..=95 {
            assert!(!binding_is_valid(LinkBinding::new(3, value)));
        }
        for value in [18, 21] {
            assert!(!binding_is_valid(LinkBinding::new(3, value)));
        }
        for binding in bindings {
            assert!(!binding_is_valid(binding));
            assert_eq!(binding_for_link(binding), binding);
            assert_eq!(action_from_binding(binding), Action::NoAction);
        }
    }

    #[test]
    fn link_function_contract_accepts_only_supported_values() {
        for value in [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 16, 17, 19, 32, 33, 34, 64, 79, 96, 110,
        ] {
            assert!(binding_is_valid(LinkBinding::new(3, value)));
        }
    }

    #[test]
    fn out_of_range_binding_lookup_returns_none() {
        let map = LinkKeymap::default();
        assert_eq!(map.binding(LINK_LAYERS, 0), None);
        assert_eq!(map.binding(0, KEY_COUNT), None);
    }

    #[test]
    fn normal_bindings_and_resets_clear_only_coordinate_hotkeys() {
        let mut map = LinkKeymap::default();
        let bound = HotkeySlot {
            empty: false,
            row: 3,
            col: 7,
            layer: 1,
            modifiers: 1,
            key: 0x06,
        };
        let other_layer = HotkeySlot { layer: 2, ..bound };
        let unbound = HotkeySlot {
            row: u8::MAX,
            col: u8::MAX,
            layer: u8::MAX,
            ..bound
        };
        assert!(map.set_hotkey(0, bound));
        assert!(map.set_hotkey(1, other_layer));
        assert!(map.set_hotkey(2, unbound));

        assert!(map.set_matrix_binding(1, 3, 7, LinkBinding::new(0, 0x04)));
        assert_eq!(map.hotkey(0), Some(HotkeySlot::EMPTY));
        assert_eq!(map.hotkey(1), Some(other_layer));
        assert_eq!(map.hotkey(2), Some(unbound));

        assert!(map.reset_layer(2));
        assert_eq!(map.hotkey(1), Some(HotkeySlot::EMPTY));
        assert_eq!(map.hotkey(2), Some(unbound));

        assert!(map.set_hotkey(3, bound));
        map.reset_all();
        assert_eq!(map.hotkey(3), Some(HotkeySlot::EMPTY));
        assert_eq!(map.hotkey(2), Some(unbound));
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
        let raw = raw_from_matrix(3, 7).unwrap();
        assert_eq!(map.binding(2, raw), map.binding(0, raw));
        assert_eq!(map.matrix_binding(2, 3, 7), Some(LinkBinding::new(3, 64)));
    }
}
