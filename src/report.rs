use crate::keymap::{Action, KEY_COUNT, base_action, function_action};

pub const FIRST_BITMAP_USAGE: u8 = 0x04;
#[cfg(feature = "layout-jis")]
pub const LAST_BITMAP_USAGE: u8 = 0x8a;
#[cfg(not(feature = "layout-jis"))]
pub const LAST_BITMAP_USAGE: u8 = 0x73;
pub const KEY_BITMAP_BITS: u8 = LAST_BITMAP_USAGE - FIRST_BITMAP_USAGE + 1;
pub const KEY_BITMAP_BYTES: usize = KEY_BITMAP_BITS.div_ceil(8) as usize;
pub const KEYBOARD_REPORT_BYTES: usize = 2 + KEY_BITMAP_BYTES;
pub const HOLD_MS: u64 = 1_000;
pub const BATTERY_HOLD_MS: u64 = 3_000;
pub const DFU_HOLD_MS: u64 = 3_000;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct KeyboardReport {
    pub modifiers: u8,
    pub reserved: u8,
    pub keys: [u8; KEY_BITMAP_BYTES],
}

impl KeyboardReport {
    pub fn as_bytes(&self) -> &[u8; KEYBOARD_REPORT_BYTES] {
        unsafe { &*(self as *const Self as *const [u8; KEYBOARD_REPORT_BYTES]) }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Command {
    #[default]
    None,
    ResetLeft,
    BootLeft,
    BootRight,
    ProfileSelect(u8),
    ProfilePair(u8),
    ProfileClear,
    OutputUsb,
    OutputBle,
    BacklightToggle,
    BacklightDown,
    BacklightUp,
    BatteryStatus,
    SystemSelect(u8),
    KeyTap(u8),
}

impl Command {
    const fn from_action(action: Action) -> Self {
        match action {
            Action::ResetLeft => Command::ResetLeft,
            Action::BootLeft | Action::BootRight => Command::None,
            Action::ProfileSelect(profile) => Command::ProfileSelect(profile),
            Action::ProfileClear => Command::ProfileClear,
            Action::OutputUsb => Command::OutputUsb,
            Action::OutputBle => Command::OutputBle,
            Action::BacklightToggle => Command::BacklightToggle,
            Action::BacklightDown => Command::BacklightDown,
            Action::BacklightUp => Command::BacklightUp,
            Action::BatteryStatus => Command::None,
            _ => Command::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Effects {
    pub keyboard: KeyboardReport,
    pub consumer: u16,
    pub keyboard_changed: bool,
    pub consumer_changed: bool,
    commands: [Command; 8],
    command_count: usize,
}

impl Effects {
    pub fn commands(&self) -> &[Command] {
        &self.commands[..self.command_count]
    }
}

pub struct ReportEngine {
    snapshot: u128,
    assigned: [Action; KEY_COUNT],
    default_layer: u8,
    keyboard: KeyboardReport,
    consumer: u16,
    pressed_at: [u64; KEY_COUNT],
    hold_triggered: u128,
}

impl Default for ReportEngine {
    fn default() -> Self {
        Self {
            snapshot: 0,
            assigned: [Action::Transparent; KEY_COUNT],
            default_layer: 0,
            keyboard: KeyboardReport::default(),
            consumer: 0,
            pressed_at: [0; KEY_COUNT],
            hold_triggered: 0,
        }
    }
}

impl ReportEngine {
    pub fn apply_snapshot(&mut self, snapshot: u128) -> Effects {
        self.apply_snapshot_at(snapshot, 0)
    }

    pub fn apply_snapshot_at(&mut self, snapshot: u128, now_ms: u64) -> Effects {
        self.apply_snapshot_with_at(snapshot, now_ms, |layer, raw| {
            if layer == 0 {
                base_action(raw)
            } else if layer == 1 {
                function_action(raw)
            } else {
                Action::Transparent
            }
        })
    }

    pub fn apply_snapshot_with(
        &mut self,
        snapshot: u128,
        mut resolve: impl FnMut(u8, usize) -> Action,
    ) -> Effects {
        self.apply_snapshot_with_at(snapshot, 0, &mut resolve)
    }

    pub fn apply_snapshot_with_at(
        &mut self,
        snapshot: u128,
        now_ms: u64,
        mut resolve: impl FnMut(u8, usize) -> Action,
    ) -> Effects {
        let snapshot = snapshot & ((1_u128 << KEY_COUNT) - 1);
        let changed = self.snapshot ^ snapshot;
        let mut active_layer = self.default_layer;
        for raw in 0..KEY_COUNT {
            if snapshot & (1_u128 << raw) == 0 {
                continue;
            }
            let action = if self.snapshot & (1_u128 << raw) != 0 {
                self.assigned[raw]
            } else {
                resolve(0, raw)
            };
            match action {
                Action::Fn => active_layer = active_layer.max(1),
                Action::LayerMomentary(layer) => active_layer = active_layer.max(layer),
                _ => {}
            }
        }
        let mut commands = [Command::None; 8];
        let mut command_count = 0;

        for raw in 0..KEY_COUNT {
            let bit = 1_u128 << raw;
            if changed & bit == 0 {
                continue;
            }
            if snapshot & bit == 0 {
                if self.hold_triggered & bit == 0 {
                    let command = match self.assigned[raw] {
                        Action::ProfileShortcut(profile) => {
                            if now_ms.saturating_sub(self.pressed_at[raw]) >= HOLD_MS {
                                Command::ProfilePair(profile)
                            } else {
                                Command::ProfileSelect(profile)
                            }
                        }
                        Action::BatteryStatus
                            if now_ms.saturating_sub(self.pressed_at[raw]) >= BATTERY_HOLD_MS =>
                        {
                            Command::BatteryStatus
                        }
                        Action::BootLeft
                            if now_ms.saturating_sub(self.pressed_at[raw]) >= DFU_HOLD_MS =>
                        {
                            Command::BootLeft
                        }
                        Action::BootRight
                            if now_ms.saturating_sub(self.pressed_at[raw]) >= DFU_HOLD_MS =>
                        {
                            Command::BootRight
                        }
                        Action::SystemShortcut { system, key } => {
                            if now_ms.saturating_sub(self.pressed_at[raw]) >= HOLD_MS {
                                Command::SystemSelect(system)
                            } else {
                                Command::KeyTap(key)
                            }
                        }
                        _ => Command::None,
                    };
                    if command != Command::None && command_count < commands.len() {
                        commands[command_count] = command;
                        command_count += 1;
                    }
                }
                self.assigned[raw] = Action::Transparent;
                self.hold_triggered &= !bit;
                continue;
            }

            let base = resolve(0, raw);
            let mut action = if matches!(base, Action::Fn | Action::LayerMomentary(_)) {
                base
            } else if active_layer != 0 {
                match resolve(active_layer, raw) {
                    Action::Transparent => base,
                    layer_action => layer_action,
                }
            } else {
                base
            };
            if let Action::LayerToggle(layer) = action {
                self.default_layer = layer;
                action = Action::NoAction;
            }
            self.assigned[raw] = action;
            self.pressed_at[raw] = now_ms;

            let command = Command::from_action(action);
            if command != Command::None && command_count < commands.len() {
                commands[command_count] = command;
                command_count += 1;
            }
        }

        for raw in 0..KEY_COUNT {
            let bit = 1_u128 << raw;
            if snapshot & bit == 0 || self.hold_triggered & bit != 0 {
                continue;
            }
            let command = match self.assigned[raw] {
                Action::ProfileShortcut(profile)
                    if now_ms.saturating_sub(self.pressed_at[raw]) >= HOLD_MS =>
                {
                    Command::ProfilePair(profile)
                }
                Action::BatteryStatus
                    if now_ms.saturating_sub(self.pressed_at[raw]) >= BATTERY_HOLD_MS =>
                {
                    Command::BatteryStatus
                }
                Action::BootLeft if now_ms.saturating_sub(self.pressed_at[raw]) >= DFU_HOLD_MS => {
                    Command::BootLeft
                }
                Action::BootRight if now_ms.saturating_sub(self.pressed_at[raw]) >= DFU_HOLD_MS => {
                    Command::BootRight
                }
                Action::SystemShortcut { system, .. }
                    if now_ms.saturating_sub(self.pressed_at[raw]) >= HOLD_MS =>
                {
                    Command::SystemSelect(system)
                }
                _ => Command::None,
            };
            if command != Command::None {
                if command_count < commands.len() {
                    commands[command_count] = command;
                    command_count += 1;
                }
                self.hold_triggered |= bit;
            }
        }

        self.snapshot = snapshot;
        let old_keyboard = self.keyboard;
        let old_consumer = self.consumer;
        self.rebuild_reports();

        Effects {
            keyboard: self.keyboard,
            consumer: self.consumer,
            keyboard_changed: self.keyboard != old_keyboard,
            consumer_changed: self.consumer != old_consumer,
            commands,
            command_count,
        }
    }

    fn rebuild_reports(&mut self) {
        let mut keyboard = KeyboardReport::default();
        let mut consumer = 0;
        for raw in 0..KEY_COUNT {
            if self.snapshot & (1_u128 << raw) == 0 {
                continue;
            }
            match self.assigned[raw] {
                Action::Key(usage @ 0xe0..=0xe7) => keyboard.modifiers |= 1 << (usage - 0xe0),
                Action::Key(usage @ FIRST_BITMAP_USAGE..=LAST_BITMAP_USAGE) => {
                    let index = (usage - FIRST_BITMAP_USAGE) as usize;
                    keyboard.keys[index / 8] |= 1 << (index & 7);
                }
                Action::Chord { modifiers, key } => {
                    keyboard.modifiers |= modifiers;
                    if (FIRST_BITMAP_USAGE..=LAST_BITMAP_USAGE).contains(&key) {
                        let index = (key - FIRST_BITMAP_USAGE) as usize;
                        keyboard.keys[index / 8] |= 1 << (index & 7);
                    }
                }
                Action::Consumer(usage) if consumer == 0 => consumer = usage,
                _ => {}
            }
        }
        self.keyboard = keyboard;
        self.consumer = consumer;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::{LEFT_FN_RAW, LEFT_KEY_COUNT, RIGHT_FN_RAW};

    fn key_is_set(report: &KeyboardReport, usage: u8) -> bool {
        let index = (usage - FIRST_BITMAP_USAGE) as usize;
        report.keys[index / 8] & (1 << (index & 7)) != 0
    }

    fn raw_with_base(action: Action) -> usize {
        (0..KEY_COUNT)
            .find(|raw| base_action(*raw) == action)
            .expect("selected layout contains the base action")
    }

    fn raw_with_function(action: Action) -> usize {
        (0..KEY_COUNT)
            .find(|raw| function_action(*raw) == action)
            .expect("selected layout contains the function action")
    }

    #[test]
    fn left_and_right_keys_share_one_report() {
        let mut engine = ReportEngine::default();
        let left = raw_with_base(Action::Key(0x3a));
        let right = (LEFT_KEY_COUNT..KEY_COUNT)
            .find(|raw| base_action(*raw) == Action::Key(0x40))
            .expect("selected layout contains right F7");
        let effects = engine.apply_snapshot((1_u128 << left) | (1_u128 << right));
        assert!(key_is_set(&effects.keyboard, 0x3a));
        assert!(key_is_set(&effects.keyboard, 0x40));
    }

    #[test]
    fn either_fn_activates_the_same_layer_even_in_one_snapshot() {
        for fn_raw in [LEFT_FN_RAW, RIGHT_FN_RAW] {
            let mut engine = ReportEngine::default();
            let effects = engine.apply_snapshot((1_u128 << fn_raw) | (1_u128 << 8));
            assert!(effects.commands().is_empty());
            assert_eq!(effects.keyboard, KeyboardReport::default());
            let effects = engine.apply_snapshot_at(1_u128 << fn_raw, 100);
            assert_eq!(effects.commands(), &[Command::ProfileSelect(0)]);
        }
    }

    #[test]
    fn profile_shortcut_taps_select_and_holds_pair_once() {
        let mut engine = ReportEngine::default();
        let keys = (1_u128 << LEFT_FN_RAW) | (1_u128 << 8);
        assert!(engine.apply_snapshot_at(keys, 10).commands().is_empty());
        assert!(engine.apply_snapshot_at(keys, 1_009).commands().is_empty());
        assert_eq!(
            engine.apply_snapshot_at(keys, 1_010).commands(),
            &[Command::ProfilePair(0)]
        );
        assert!(engine.apply_snapshot_at(keys, 2_000).commands().is_empty());
        assert!(
            engine
                .apply_snapshot_at(1_u128 << LEFT_FN_RAW, 2_001)
                .commands()
                .is_empty()
        );

        let keys = (1_u128 << LEFT_FN_RAW) | (1_u128 << 9);
        assert!(engine.apply_snapshot_at(keys, 3_000).commands().is_empty());
        assert_eq!(
            engine
                .apply_snapshot_at(1_u128 << LEFT_FN_RAW, 3_999)
                .commands(),
            &[Command::ProfileSelect(1)]
        );
    }

    #[test]
    fn battery_status_requires_three_seconds_and_triggers_once() {
        for fn_raw in [LEFT_FN_RAW, RIGHT_FN_RAW] {
            let mut engine = ReportEngine::default();
            let keys = (1_u128 << fn_raw) | (1_u128 << raw_with_base(Action::Key(0x0c)));
            assert!(engine.apply_snapshot_at(keys, 10).commands().is_empty());
            assert!(engine.apply_snapshot_at(keys, 3_009).commands().is_empty());
            assert_eq!(
                engine.apply_snapshot_at(keys, 3_010).commands(),
                &[Command::BatteryStatus]
            );
            assert!(engine.apply_snapshot_at(keys, 4_000).commands().is_empty());
            assert!(engine.apply_snapshot_at(0, 4_001).commands().is_empty());
        }
    }

    #[test]
    fn system_shortcuts_tap_letters_and_hold_to_select_the_os() {
        let mut engine = ReportEngine::default();
        let keys = (1_u128 << LEFT_FN_RAW) | (1_u128 << raw_with_base(Action::Key(0x11)));
        assert!(engine.apply_snapshot_at(keys, 10).commands().is_empty());
        assert_eq!(
            engine
                .apply_snapshot_at(1_u128 << LEFT_FN_RAW, 1_009)
                .commands(),
            &[Command::KeyTap(0x11)]
        );

        let keys = (1_u128 << RIGHT_FN_RAW) | (1_u128 << raw_with_base(Action::Key(0x10)));
        assert!(engine.apply_snapshot_at(keys, 2_000).commands().is_empty());
        assert_eq!(
            engine.apply_snapshot_at(keys, 3_000).commands(),
            &[Command::SystemSelect(0)]
        );
        assert!(engine.apply_snapshot_at(keys, 4_000).commands().is_empty());
        assert!(engine.apply_snapshot_at(0, 4_001).commands().is_empty());
    }

    #[test]
    fn transparent_function_keys_keep_their_base_action() {
        let mut engine = ReportEngine::default();
        let q = raw_with_base(Action::Key(0x14));
        let effects = engine.apply_snapshot((1_u128 << LEFT_FN_RAW) | (1_u128 << q));
        assert!(key_is_set(&effects.keyboard, 0x14));
    }

    #[test]
    fn action_selected_on_press_survives_fn_release() {
        let mut engine = ReportEngine::default();
        let mute = raw_with_function(Action::Consumer(0x00e2));
        engine.apply_snapshot((1_u128 << LEFT_FN_RAW) | (1_u128 << mute));
        let effects = engine.apply_snapshot(1_u128 << mute);
        assert_eq!(effects.consumer, 0x00e2);
        assert!(!effects.consumer_changed);
        assert_eq!(engine.apply_snapshot(0).consumer, 0);
    }

    #[test]
    fn modifiers_are_encoded_separately() {
        let mut engine = ReportEngine::default();
        let left_control = raw_with_base(Action::Key(0xe0));
        let right_alt = raw_with_base(Action::Key(0xe6));
        let effects = engine.apply_snapshot((1_u128 << left_control) | (1_u128 << right_alt));
        assert_eq!(effects.keyboard.modifiers, (1 << 0) | (1 << 6));
    }

    #[test]
    fn dfu_shortcuts_require_three_seconds_and_trigger_once() {
        for (fn_raw, key_raw, command) in [
            (
                LEFT_FN_RAW,
                raw_with_base(Action::Key(0x22)),
                Command::BootLeft,
            ),
            (
                RIGHT_FN_RAW,
                raw_with_base(Action::Key(0x27)),
                Command::BootRight,
            ),
        ] {
            let mut engine = ReportEngine::default();
            let keys = (1_u128 << fn_raw) | (1_u128 << key_raw);
            assert!(engine.apply_snapshot_at(keys, 10).commands().is_empty());
            assert!(engine.apply_snapshot_at(keys, 3_009).commands().is_empty());
            assert_eq!(engine.apply_snapshot_at(keys, 3_010).commands(), &[command]);
            assert!(engine.apply_snapshot_at(keys, 4_000).commands().is_empty());
        }
    }

    #[test]
    fn keyboard_report_has_no_padding() {
        assert_eq!(
            core::mem::size_of::<KeyboardReport>(),
            KEYBOARD_REPORT_BYTES
        );
        assert_eq!(
            KeyboardReport::default().as_bytes().len(),
            KEYBOARD_REPORT_BYTES
        );
        assert!(
            KeyboardReport::default()
                .as_bytes()
                .iter()
                .all(|byte| *byte == 0)
        );
    }

    #[cfg(feature = "layout-jis")]
    #[test]
    fn jis_extended_usages_reach_the_keyboard_report() {
        assert_eq!(
            (LAST_BITMAP_USAGE, KEY_BITMAP_BITS, KEYBOARD_REPORT_BYTES),
            (0x8a, 135, 19)
        );
        for usage in [0x87, 0x89, 0x8a] {
            let mut engine = ReportEngine::default();
            let raw = raw_with_base(Action::Key(usage));
            let report = engine.apply_snapshot(1_u128 << raw).keyboard;
            assert!(
                key_is_set(&report, usage),
                "JIS usage {usage:#04x} was dropped"
            );
        }
    }

    #[cfg(not(feature = "layout-jis"))]
    #[test]
    fn standard_layouts_keep_the_existing_report_shape() {
        assert_eq!(
            (LAST_BITMAP_USAGE, KEY_BITMAP_BITS, KEYBOARD_REPORT_BYTES),
            (0x73, 112, 16)
        );
    }

    #[test]
    fn configurable_layers_and_chords_apply_to_reports() {
        let mut engine = ReportEngine::default();
        let effects = engine.apply_snapshot_with((1_u128 << 32) | (1_u128 << 37), |layer, raw| {
            match (layer, raw) {
                (0, 32) => Action::LayerMomentary(1),
                (1, 37) => Action::Chord {
                    modifiers: 1 << 3,
                    key: 0x07,
                },
                _ => Action::Transparent,
            }
        });
        assert_eq!(effects.keyboard.modifiers, 1 << 3);
        assert!(key_is_set(&effects.keyboard, 0x07));
    }
}
