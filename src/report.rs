use crate::keymap::{Action, KEY_COUNT, base_action, function_action};

pub const KEY_BITMAP_BYTES: usize = 14;
const FIRST_BITMAP_USAGE: u8 = 0x04;
const LAST_BITMAP_USAGE: u8 = 0x73;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct KeyboardReport {
    pub modifiers: u8,
    pub reserved: u8,
    pub keys: [u8; KEY_BITMAP_BYTES],
}

impl KeyboardReport {
    pub fn as_bytes(&self) -> &[u8; 16] {
        unsafe { &*(self as *const Self as *const [u8; 16]) }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Command {
    #[default]
    None,
    ResetLeft,
    BootRight,
    ProfileSelect(u8),
    ProfileClear,
    OutputUsb,
    OutputBle,
}

impl Command {
    const fn from_action(action: Action) -> Self {
        match action {
            Action::ResetLeft => Command::ResetLeft,
            Action::BootRight => Command::BootRight,
            Action::ProfileSelect(profile) => Command::ProfileSelect(profile),
            Action::ProfileClear => Command::ProfileClear,
            Action::OutputUsb => Command::OutputUsb,
            Action::OutputBle => Command::OutputBle,
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
}

impl Default for ReportEngine {
    fn default() -> Self {
        Self {
            snapshot: 0,
            assigned: [Action::Transparent; KEY_COUNT],
            default_layer: 0,
            keyboard: KeyboardReport::default(),
            consumer: 0,
        }
    }
}

impl ReportEngine {
    pub fn apply_snapshot(&mut self, snapshot: u128) -> Effects {
        self.apply_snapshot_with(snapshot, |layer, raw| {
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
                self.assigned[raw] = Action::Transparent;
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

            let command = Command::from_action(action);
            if command != Command::None && command_count < commands.len() {
                commands[command_count] = command;
                command_count += 1;
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
    use crate::keymap::{LEFT_FN_RAW, RIGHT_FN_RAW};

    fn key_is_set(report: &KeyboardReport, usage: u8) -> bool {
        let index = (usage - FIRST_BITMAP_USAGE) as usize;
        report.keys[index / 8] & (1 << (index & 7)) != 0
    }

    #[test]
    fn left_and_right_keys_share_one_report() {
        let mut engine = ReportEngine::default();
        let effects = engine.apply_snapshot((1 << 1) | (1_u128 << 37));
        assert!(key_is_set(&effects.keyboard, 0x3a));
        assert!(key_is_set(&effects.keyboard, 0x40));
    }

    #[test]
    fn either_fn_activates_the_same_layer_even_in_one_snapshot() {
        for fn_raw in [LEFT_FN_RAW, RIGHT_FN_RAW] {
            let mut engine = ReportEngine::default();
            let effects = engine.apply_snapshot((1_u128 << fn_raw) | (1_u128 << 8));
            assert_eq!(effects.commands(), &[Command::ProfileSelect(0)]);
            assert_eq!(effects.keyboard, KeyboardReport::default());
        }
    }

    #[test]
    fn transparent_function_keys_keep_their_base_action() {
        let mut engine = ReportEngine::default();
        let effects = engine.apply_snapshot((1_u128 << LEFT_FN_RAW) | (1_u128 << 14));
        assert!(key_is_set(&effects.keyboard, 0x2b));
    }

    #[test]
    fn action_selected_on_press_survives_fn_release() {
        let mut engine = ReportEngine::default();
        engine.apply_snapshot((1_u128 << LEFT_FN_RAW) | (1_u128 << 40));
        let effects = engine.apply_snapshot(1_u128 << 40);
        assert_eq!(effects.consumer, 0x00e2);
        assert!(!effects.consumer_changed);
        assert_eq!(engine.apply_snapshot(0).consumer, 0);
    }

    #[test]
    fn modifiers_are_encoded_separately() {
        let mut engine = ReportEngine::default();
        let effects = engine.apply_snapshot((1_u128 << 33) | (1_u128 << 80));
        assert_eq!(effects.keyboard.modifiers, (1 << 0) | (1 << 6));
    }

    #[test]
    fn right_delete_requests_right_bootloader() {
        let mut engine = ReportEngine::default();
        let effects = engine.apply_snapshot((1_u128 << RIGHT_FN_RAW) | (1_u128 << 68));
        assert_eq!(effects.commands(), &[Command::BootRight]);
    }

    #[test]
    fn keyboard_report_has_no_padding() {
        assert_eq!(core::mem::size_of::<KeyboardReport>(), 16);
        assert_eq!(KeyboardReport::default().as_bytes(), &[0; 16]);
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
