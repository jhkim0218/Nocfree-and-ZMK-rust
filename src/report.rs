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
pub const CAPS_TAPPING_TERM_MS: u64 = 2_000;

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
#[repr(C)]
pub struct PointerReport {
    pub buttons: u8,
    pub x: i8,
    pub y: i8,
    pub wheel: i8,
}

impl PointerReport {
    pub fn as_bytes(&self) -> &[u8; POINTER_REPORT_BYTES] {
        unsafe { &*(self as *const Self as *const [u8; POINTER_REPORT_BYTES]) }
    }

    const fn has_motion(self) -> bool {
        self.x != 0 || self.y != 0 || self.wheel != 0
    }
}

pub const POINTER_REPORT_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReportFrame {
    pub keyboard: KeyboardReport,
    pub consumer: u16,
    pub pointer: PointerReport,
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
            Action::SystemSelect(system) => Command::SystemSelect(system),
            _ => Command::None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Effects {
    pub keyboard: KeyboardReport,
    pub consumer: u16,
    pub pointer: PointerReport,
    pub keyboard_changed: bool,
    pub consumer_changed: bool,
    pub pointer_changed: bool,
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
    pointer: PointerReport,
    pressed_at: [u64; KEY_COUNT],
    hold_triggered: u128,
    caps_layer_held: u128,
}

impl Default for ReportEngine {
    fn default() -> Self {
        Self {
            snapshot: 0,
            assigned: [Action::Transparent; KEY_COUNT],
            default_layer: 0,
            keyboard: KeyboardReport::default(),
            consumer: 0,
            pointer: PointerReport::default(),
            pressed_at: [0; KEY_COUNT],
            hold_triggered: 0,
            caps_layer_held: 0,
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
                Action::CapsLayer { layer, .. } if self.caps_layer_should_hold(raw, now_ms) => {
                    self.caps_layer_held |= 1_u128 << raw;
                    active_layer = active_layer.max(layer);
                }
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
                let caps_layer_held = self.caps_layer_should_hold(raw, now_ms);
                if self.hold_triggered & bit == 0 {
                    let command = match self.assigned[raw] {
                        Action::CapsLayer { tap, .. } if !caps_layer_held => Command::KeyTap(tap),
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
                self.caps_layer_held &= !bit;
                continue;
            }

            let base = resolve(0, raw);
            let mut action = if matches!(
                base,
                Action::Fn | Action::LayerMomentary(_) | Action::CapsLayer { .. }
            ) {
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
        let old_pointer = self.pointer;
        self.rebuild_reports();

        Effects {
            keyboard: self.keyboard,
            consumer: self.consumer,
            pointer: self.pointer,
            keyboard_changed: self.keyboard != old_keyboard,
            consumer_changed: self.consumer != old_consumer,
            pointer_changed: self.pointer != old_pointer || self.pointer.has_motion(),
            commands,
            command_count,
        }
    }

    fn rebuild_reports(&mut self) {
        let mut keyboard = KeyboardReport::default();
        let mut consumer = 0;
        let mut pointer = PointerReport::default();
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
                Action::PointerMove { x, y } => {
                    pointer.x = pointer.x.saturating_add(x);
                    pointer.y = pointer.y.saturating_add(y);
                }
                Action::PointerWheel(wheel) => pointer.wheel = pointer.wheel.saturating_add(wheel),
                Action::PointerButton(button @ 1..=3) => pointer.buttons |= 1 << (button - 1),
                _ => {}
            }
        }
        self.keyboard = keyboard;
        self.consumer = consumer;
        self.pointer = pointer;
    }

    fn caps_layer_should_hold(&self, raw: usize, now_ms: u64) -> bool {
        let bit = 1_u128 << raw;
        self.snapshot & bit != 0
            && (self.caps_layer_held & bit != 0
                || now_ms.saturating_sub(self.pressed_at[raw]) >= CAPS_TAPPING_TERM_MS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keymap::{LEFT_FN_RAW, LEFT_KEY_COUNT, RIGHT_FN_RAW, layer2_action};

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
    fn direct_system_selection_is_immediate() {
        let mut engine = ReportEngine::default();
        let effects = engine.apply_snapshot_with_at(1, 10, |_, raw| match raw {
            0 => Action::SystemSelect(1),
            _ => Action::Transparent,
        });
        assert_eq!(effects.commands(), &[Command::SystemSelect(1)]);
        assert!(engine.apply_snapshot_at(1, 1_010).commands().is_empty());
        assert!(engine.apply_snapshot_at(0, 1_011).commands().is_empty());
    }

    fn caps_layer_action(layer: u8, raw: usize, caps_lock: usize) -> Action {
        if raw == caps_lock && layer <= 1 {
            Action::CapsLayer {
                tap: 0x39,
                layer: 2,
            }
        } else if layer == 0 {
            base_action(raw)
        } else if layer == 2 {
            layer2_action(raw)
        } else {
            Action::Transparent
        }
    }

    #[test]
    fn caps_hold_layer2_taps_caps_lock_and_activates_after_term() {
        let caps_lock = raw_with_base(Action::Key(0x39));
        let a = raw_with_base(Action::Key(0x04));
        let mut engine = ReportEngine::default();

        assert!(
            engine
                .apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
                    caps_layer_action(layer, raw, caps_lock)
                })
                .commands()
                .is_empty()
        );
        assert_eq!(
            engine
                .apply_snapshot_with_at(0, CAPS_TAPPING_TERM_MS - 1, |layer, raw| {
                    caps_layer_action(layer, raw, caps_lock)
                })
                .commands(),
            &[Command::KeyTap(0x39)]
        );

        let mut engine = ReportEngine::default();
        engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
            caps_layer_action(layer, raw, caps_lock)
        });
        let effects = engine.apply_snapshot_with_at(
            (1_u128 << caps_lock) | (1_u128 << a),
            CAPS_TAPPING_TERM_MS,
            |layer, raw| caps_layer_action(layer, raw, caps_lock),
        );
        assert!(key_is_set(&effects.keyboard, 0x50));
        assert!(effects.commands().is_empty());
        let effects =
            engine.apply_snapshot_with_at(1_u128 << a, CAPS_TAPPING_TERM_MS + 1, |layer, raw| {
                caps_layer_action(layer, raw, caps_lock)
            });
        assert!(key_is_set(&effects.keyboard, 0x50));
        assert!(effects.commands().is_empty());
    }

    #[test]
    fn caps_hold_layer2_requires_two_seconds_on_every_press() {
        let caps_lock = raw_with_base(Action::Key(0x39));
        let i = raw_with_base(Action::Key(0x0c));
        let caps = 1_u128 << caps_lock;
        let keys = caps | (1_u128 << i);
        let resolve = |layer, raw| caps_layer_action(layer, raw, caps_lock);
        let mut engine = ReportEngine::default();

        for start in [10_000, 20_000] {
            // Uptime and other keys must not turn a short Caps Lock press into a hold.
            let effects = engine.apply_snapshot_with_at(keys, start, resolve);
            assert!(key_is_set(&effects.keyboard, 0x0c));
            assert_eq!(effects.pointer, PointerReport::default());
            engine.apply_snapshot_with_at(caps, start + 1, resolve);
            let effects = engine.apply_snapshot_with_at(keys, start + 1_999, resolve);
            assert!(key_is_set(&effects.keyboard, 0x0c));
            assert_eq!(effects.pointer, PointerReport::default());
            assert_eq!(
                engine
                    .apply_snapshot_with_at(0, start + 1_999, resolve)
                    .commands(),
                &[Command::KeyTap(0x39)]
            );
        }

        engine.apply_snapshot_with_at(caps, 30_000, resolve);
        let effects = engine.apply_snapshot_with_at(keys, 32_000, resolve);
        assert_eq!(effects.pointer.y, -8);
        assert!(!key_is_set(&effects.keyboard, 0x0c));
        assert!(
            engine
                .apply_snapshot_with_at(0, 32_001, resolve)
                .commands()
                .is_empty()
        );
        let effects = engine.apply_snapshot_with_at(1_u128 << i, 32_002, resolve);
        assert!(key_is_set(&effects.keyboard, 0x0c));
        assert_eq!(effects.pointer, PointerReport::default());
    }

    #[test]
    fn caps_hold_layer2_release_after_two_seconds_does_not_tap_without_a_tick() {
        let caps_lock = raw_with_base(Action::Key(0x39));
        for elapsed in [2_000, 2_500] {
            let mut engine = ReportEngine::default();
            let resolve = |layer, raw| caps_layer_action(layer, raw, caps_lock);
            engine.apply_snapshot_with_at(1_u128 << caps_lock, 10_000, resolve);
            assert!(
                engine
                    .apply_snapshot_with_at(0, 10_000 + elapsed, resolve)
                    .commands()
                    .is_empty()
            );
        }
    }

    #[test]
    fn caps_hold_layer2_maps_browser_navigation() {
        let caps_lock = raw_with_base(Action::Key(0x39));
        let q = raw_with_base(Action::Key(0x14));
        let z = raw_with_base(Action::Key(0x1d));
        let mut engine = ReportEngine::default();
        engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
            caps_layer_action(layer, raw, caps_lock)
        });
        let effects = engine.apply_snapshot_with_at(
            (1_u128 << caps_lock) | (1_u128 << q),
            CAPS_TAPPING_TERM_MS,
            |layer, raw| caps_layer_action(layer, raw, caps_lock),
        );
        assert_eq!(effects.consumer, 0x0224);

        let mut engine = ReportEngine::default();
        engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
            caps_layer_action(layer, raw, caps_lock)
        });
        let effects = engine.apply_snapshot_with_at(
            (1_u128 << caps_lock) | (1_u128 << z),
            CAPS_TAPPING_TERM_MS,
            |layer, raw| caps_layer_action(layer, raw, caps_lock),
        );
        assert!(key_is_set(&effects.keyboard, 0x4b));
    }

    #[test]
    fn caps_hold_layer2_moves_scrolls_and_clicks() {
        let caps_lock = raw_with_base(Action::Key(0x39));
        let i = raw_with_base(Action::Key(0x0c));
        let j = raw_with_base(Action::Key(0x0d));
        let u = raw_with_base(Action::Key(0x18));
        let m = raw_with_base(Action::Key(0x10));
        let c = raw_with_base(Action::Key(0x06));
        let v = raw_with_base(Action::Key(0x19));
        let b = raw_with_base(Action::Key(0x05));
        let mut engine = ReportEngine::default();
        engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
            caps_layer_action(layer, raw, caps_lock)
        });
        let effects = engine.apply_snapshot_with_at(
            (1_u128 << caps_lock) | (1_u128 << i),
            CAPS_TAPPING_TERM_MS,
            |layer, raw| caps_layer_action(layer, raw, caps_lock),
        );
        assert_eq!(
            effects.pointer,
            PointerReport {
                buttons: 0,
                x: 0,
                y: -8,
                wheel: 0
            }
        );
        assert!(effects.pointer_changed);
        assert!(
            engine
                .apply_snapshot_with_at(
                    (1_u128 << caps_lock) | (1_u128 << i),
                    CAPS_TAPPING_TERM_MS + 20,
                    |layer, raw| { caps_layer_action(layer, raw, caps_lock) }
                )
                .pointer_changed
        );

        let mut engine = ReportEngine::default();
        engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
            caps_layer_action(layer, raw, caps_lock)
        });
        let effects = engine.apply_snapshot_with_at(
            (1_u128 << caps_lock) | (1_u128 << j) | (1_u128 << c),
            CAPS_TAPPING_TERM_MS,
            |layer, raw| caps_layer_action(layer, raw, caps_lock),
        );
        assert_eq!(
            effects.pointer,
            PointerReport {
                buttons: 1,
                x: -8,
                y: 0,
                wheel: 0
            }
        );
        for (raw, expected) in [(u, 1), (m, -1)] {
            let mut engine = ReportEngine::default();
            engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
                caps_layer_action(layer, raw, caps_lock)
            });
            let effects = engine.apply_snapshot_with_at(
                (1_u128 << caps_lock) | (1_u128 << raw),
                CAPS_TAPPING_TERM_MS,
                |layer, raw| caps_layer_action(layer, raw, caps_lock),
            );
            assert_eq!(effects.pointer.wheel, expected);
        }
        for (raw, expected) in [(c, 1), (v, 2), (b, 4)] {
            let mut engine = ReportEngine::default();
            engine.apply_snapshot_with_at(1_u128 << caps_lock, 0, |layer, raw| {
                caps_layer_action(layer, raw, caps_lock)
            });
            let effects = engine.apply_snapshot_with_at(
                (1_u128 << caps_lock) | (1_u128 << raw),
                CAPS_TAPPING_TERM_MS,
                |layer, raw| caps_layer_action(layer, raw, caps_lock),
            );
            assert_eq!(effects.pointer.buttons, expected);
        }
    }

    #[test]
    fn system_shortcuts_keep_their_mn_tap_and_one_second_hold_behavior() {
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
        assert!(engine.apply_snapshot_at(keys, 2_999).commands().is_empty());
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
