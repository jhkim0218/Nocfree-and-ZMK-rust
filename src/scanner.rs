use crate::keymap::{LEFT_KEY_COUNT, RIGHT_KEY_COUNT};

pub const EXPANDER_ADDRESSES: [u8; 3] = [0x20, 0x22, 0x24];
pub const ACTIVE_SCAN_MS: u16 = 3;
pub const IDLE_SCAN_MS: u16 = 10;
pub const IDLE_SAFETY_SCAN_MS: u16 = 250;
pub const DEBOUNCE_PRESS_MS: u16 = 5;
pub const DEBOUNCE_RELEASE_MS: u16 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Half {
    Left,
    Right,
}

const RIGHT_STATE_TAG: u64 = 1_u64 << 63;

pub const fn encode_half_state(half: Half, pressed: u64) -> u64 {
    match half {
        Half::Left => pressed,
        Half::Right => pressed | RIGHT_STATE_TAG,
    }
}

pub const fn decode_half_state(encoded: u64) -> (Half, u64) {
    if encoded & RIGHT_STATE_TAG == 0 {
        (Half::Left, encoded)
    } else {
        (Half::Right, encoded & !RIGHT_STATE_TAG)
    }
}

#[derive(Default)]
pub struct SnapshotMerger {
    left: u64,
    right: u64,
}

impl SnapshotMerger {
    pub const fn update(&mut self, half: Half, pressed: u64) -> u128 {
        match half {
            Half::Left => self.left = pressed,
            Half::Right => self.right = pressed,
        }
        global_snapshot(self.left, self.right)
    }
}

impl Half {
    pub const fn row_counts(self) -> [u8; 6] {
        match self {
            Half::Left => [7, 7, 6, 6, 6, 5],
            Half::Right => [8, 8, 8, 8, 8, 7],
        }
    }

    pub const fn key_count(self) -> usize {
        match self {
            Half::Left => LEFT_KEY_COUNT,
            Half::Right => RIGHT_KEY_COUNT,
        }
    }
}

pub fn decode_pressed(half: Half, port_words: [u16; 3]) -> u64 {
    let counts = half.row_counts();
    let mut pressed = 0_u64;
    let mut local = 0;
    let mut row = 0;
    while row < counts.len() {
        let base_bit = if row & 1 == 0 { 0 } else { 8 };
        let mut column = 0;
        while column < counts[row] {
            let bit = base_bit + column;
            if port_words[row / 2] & (1_u16 << bit) == 0 {
                pressed |= 1_u64 << local;
            }
            local += 1;
            column += 1;
        }
        row += 1;
    }
    pressed
}

pub const fn global_snapshot(left: u64, right: u64) -> u128 {
    left as u128 | ((right as u128) << LEFT_KEY_COUNT)
}

pub const fn failure_backoff_ms(fail_streak: u8) -> u16 {
    let shift = if fail_streak > 7 { 7 } else { fail_streak };
    let delay = (IDLE_SCAN_MS as u32) << shift;
    if delay > 1000 { 1000 } else { delay as u16 }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DebounceUpdate {
    pub pressed: u128,
    pub changed: u128,
    pub active: bool,
}

pub struct Debouncer<const N: usize> {
    pressed: u128,
    counters_ms: [u16; N],
}

impl<const N: usize> Default for Debouncer<N> {
    fn default() -> Self {
        assert!(N <= 128);
        Self {
            pressed: 0,
            counters_ms: [0; N],
        }
    }
}

impl<const N: usize> Debouncer<N> {
    pub fn update(&mut self, raw_pressed: u128, elapsed_ms: u16) -> DebounceUpdate {
        let elapsed_ms = elapsed_ms.max(1);
        let mut changed = 0;
        let mut active = false;

        for key in 0..N {
            let bit = 1_u128 << key;
            let raw = raw_pressed & bit != 0;
            let latched = self.pressed & bit != 0;
            if raw == latched {
                self.counters_ms[key] = 0;
                continue;
            }

            active = true;
            let credit = if self.counters_ms[key] == 0 {
                elapsed_ms.min(ACTIVE_SCAN_MS)
            } else {
                elapsed_ms
            };
            self.counters_ms[key] = self.counters_ms[key].saturating_add(credit);
            let threshold = if raw {
                DEBOUNCE_PRESS_MS
            } else {
                DEBOUNCE_RELEASE_MS
            };
            if self.counters_ms[key] >= threshold {
                self.pressed ^= bit;
                changed |= bit;
                self.counters_ms[key] = 0;
            }
        }

        DebounceUpdate {
            pressed: self.pressed,
            changed,
            active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn released_and_pressed_levels_are_active_low() {
        assert_eq!(decode_pressed(Half::Left, [0xffff; 3]), 0);
        assert_eq!(decode_pressed(Half::Right, [0xffff; 3]), 0);
        assert_eq!(decode_pressed(Half::Left, [0xfffe, 0xffff, 0xffff]), 1);
        assert_eq!(decode_pressed(Half::Right, [0xfffe, 0xffff, 0xffff]), 1);
    }

    #[test]
    fn every_populated_input_resolves_once() {
        for half in [Half::Left, Half::Right] {
            let counts = half.row_counts();
            let mut expected = 0;
            for row in 0..6 {
                for column in 0..counts[row] {
                    let mut words = [0xffff; 3];
                    words[row / 2] &= !(1 << ((row & 1) * 8 + column as usize));
                    assert_eq!(decode_pressed(half, words), 1_u64 << expected);
                    expected += 1;
                }
            }
            assert_eq!(expected, half.key_count());
        }
    }

    #[test]
    fn first_idle_observation_does_not_skip_debounce() {
        let mut debounce = Debouncer::<1>::default();
        let first = debounce.update(1, IDLE_SCAN_MS);
        assert_eq!(first.pressed, 0);
        assert!(first.active);
        let second = debounce.update(1, ACTIVE_SCAN_MS);
        assert_eq!(second.pressed, 1);
        assert_eq!(second.changed, 1);
    }

    #[test]
    fn bounce_resets_credit_and_release_is_debounced() {
        let mut debounce = Debouncer::<1>::default();
        debounce.update(1, 3);
        debounce.update(0, 3);
        assert_eq!(debounce.update(1, 3).pressed, 0);
        assert_eq!(debounce.update(1, 3).pressed, 1);
        assert_eq!(debounce.update(0, 10).pressed, 1);
        assert_eq!(debounce.update(0, 3).pressed, 0);
    }

    #[test]
    fn retry_backoff_caps_at_one_second() {
        assert_eq!(failure_backoff_ms(0), 10);
        assert_eq!(failure_backoff_ms(1), 20);
        assert_eq!(failure_backoff_ms(7), 1000);
        assert_eq!(failure_backoff_ms(u8::MAX), 1000);
    }

    #[test]
    fn idle_safety_scan_is_much_slower_than_the_old_poll() {
        assert_eq!(IDLE_SAFETY_SCAN_MS, 250);
        assert!(IDLE_SAFETY_SCAN_MS >= IDLE_SCAN_MS * 20);
    }

    #[test]
    fn one_tagged_stream_preserves_cross_half_arrival_order() {
        let mut merger = SnapshotMerger::default();
        let updates = [
            encode_half_state(Half::Right, 1 << 7),
            encode_half_state(Half::Left, 1 << 3),
            encode_half_state(Half::Right, (1 << 7) | (1 << 19)),
        ];
        let expected = [
            1_u128 << (LEFT_KEY_COUNT + 7),
            (1_u128 << 3) | (1_u128 << (LEFT_KEY_COUNT + 7)),
            (1_u128 << 3) | (1_u128 << (LEFT_KEY_COUNT + 7)) | (1_u128 << (LEFT_KEY_COUNT + 19)),
        ];

        for (encoded, expected) in updates.into_iter().zip(expected) {
            let (half, pressed) = decode_half_state(encoded);
            assert_eq!(merger.update(half, pressed), expected);
        }
    }
}
