use crate::keymap::{
    EXPANDER_COUNT, EXTRA_LEFT_KEYS, EXTRA_RIGHT_KEYS, LEFT_KEY_COUNT, LEFT_ROW_COUNTS,
    RIGHT_KEY_COUNT, RIGHT_ROW_COUNTS,
};

pub const ACTIVE_SCAN_MS: u16 = 3;
pub const IDLE_SCAN_MS: u16 = 10;
pub const IDLE_SAFETY_SCAN_MS: u16 = 250;
pub const DEBOUNCE_PRESS_MS: u16 = 5;
pub const DEBOUNCE_RELEASE_MS: u16 = 5;
// Builder tuning point: smaller values reduce latency, while larger values tolerate more
// cross-half transport jitter. Rebuild and flash both halves together after changing this,
// then test fast L-R-L/R-L-R sequences over both USB and BLE.
pub const REORDER_WINDOW_MS: u64 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Half {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimedSnapshot {
    pub half: Half,
    pub pressed: u64,
    pub source_micros: u64,
    pub sequence: u16,
    pub reconcile: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SequenceStatus {
    First,
    Next,
    Gap,
    Duplicate,
}

pub struct SnapshotOrderer<const N: usize> {
    pending: [Option<TimedSnapshot>; N],
    len: usize,
    last_sequence: [Option<u16>; 2],
}

impl<const N: usize> Default for SnapshotOrderer<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> SnapshotOrderer<N> {
    pub const fn new() -> Self {
        assert!(N > 0);
        Self {
            pending: [None; N],
            len: 0,
            last_sequence: [None; 2],
        }
    }

    pub fn push(&mut self, event: TimedSnapshot) -> Result<SequenceStatus, TimedSnapshot> {
        let source = event.half as usize;
        if event.reconcile {
            self.remove_half(event.half);
            self.last_sequence[source] = None;
        }

        let status = match self.last_sequence[source] {
            None => SequenceStatus::First,
            Some(previous) if event.sequence == previous => SequenceStatus::Duplicate,
            Some(previous) if event.sequence == previous.wrapping_add(1) => SequenceStatus::Next,
            Some(_) => SequenceStatus::Gap,
        };
        if status == SequenceStatus::Duplicate {
            return Ok(status);
        }
        if self.len == N {
            return Err(event);
        }
        self.last_sequence[source] = Some(event.sequence);

        let mut position = self.len;
        while position > 0 {
            let previous = self.pending[position - 1].unwrap();
            if previous.source_micros <= event.source_micros {
                break;
            }
            self.pending[position] = Some(previous);
            position -= 1;
        }
        self.pending[position] = Some(event);
        self.len += 1;
        Ok(status)
    }

    pub fn pop_ready(&mut self, now_micros: u64, window_micros: u64) -> Option<TimedSnapshot> {
        let first = self.pending[0]?;
        (first.source_micros.saturating_add(window_micros) <= now_micros)
            .then(|| self.pop_oldest().unwrap())
    }

    pub fn pop_oldest(&mut self) -> Option<TimedSnapshot> {
        let first = self.pending[0]?;
        self.len -= 1;
        for index in 0..self.len {
            self.pending[index] = self.pending[index + 1];
        }
        self.pending[self.len] = None;
        Some(first)
    }

    pub fn wait_micros(&self, now_micros: u64, window_micros: u64) -> Option<u64> {
        self.pending[0].map(|first| {
            first
                .source_micros
                .saturating_add(window_micros)
                .saturating_sub(now_micros)
        })
    }

    fn remove_half(&mut self, half: Half) {
        let mut write = 0;
        for read in 0..self.len {
            let event = self.pending[read].unwrap();
            if event.half != half {
                self.pending[write] = Some(event);
                write += 1;
            }
        }
        for index in write..self.len {
            self.pending[index] = None;
        }
        self.len = write;
    }
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
            Half::Left => LEFT_ROW_COUNTS,
            Half::Right => RIGHT_ROW_COUNTS,
        }
    }

    pub const fn key_count(self) -> usize {
        match self {
            Half::Left => LEFT_KEY_COUNT,
            Half::Right => RIGHT_KEY_COUNT,
        }
    }
}

pub fn decode_pressed(half: Half, port_words: [u16; EXPANDER_COUNT]) -> u64 {
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
    let extra = match half {
        Half::Left => EXTRA_LEFT_KEYS,
        Half::Right => EXTRA_RIGHT_KEYS,
    };
    if extra != 0 {
        let extra_word = port_words.get(3).copied().unwrap_or(u16::MAX);
        for bit in 0..extra {
            if extra_word & (1_u16 << bit) == 0 {
                pressed |= 1_u64 << local;
            }
            local += 1;
        }
    }
    debug_assert_eq!(local, half.key_count());
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
    use std::vec::Vec;

    #[test]
    fn released_and_pressed_levels_are_active_low() {
        let released = [0xffff; EXPANDER_COUNT];
        let mut first_pressed = released;
        first_pressed[0] = 0xfffe;
        assert_eq!(decode_pressed(Half::Left, released), 0);
        assert_eq!(decode_pressed(Half::Right, released), 0);
        assert_eq!(decode_pressed(Half::Left, first_pressed), 1);
        assert_eq!(decode_pressed(Half::Right, first_pressed), 1);
    }

    #[test]
    fn every_populated_input_resolves_once() {
        for half in [Half::Left, Half::Right] {
            let counts = half.row_counts();
            let mut expected = 0;
            for row in 0..6 {
                for column in 0..counts[row] {
                    let mut words = [0xffff; EXPANDER_COUNT];
                    words[row / 2] &= !(1 << ((row & 1) * 8 + column as usize));
                    assert_eq!(decode_pressed(half, words), 1_u64 << expected);
                    expected += 1;
                }
            }
            let extra = match half {
                Half::Left => EXTRA_LEFT_KEYS,
                Half::Right => EXTRA_RIGHT_KEYS,
            };
            for bit in 0..extra {
                let mut words = [0xffff; EXPANDER_COUNT];
                *words.get_mut(3).expect("extra layout expander") &= !(1 << bit);
                assert_eq!(decode_pressed(half, words), 1_u64 << expected);
                expected += 1;
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

    #[test]
    fn detects_duplicate_gap_and_reconciliation() {
        let event = |sequence, reconcile| TimedSnapshot {
            half: Half::Right,
            pressed: sequence as u64,
            source_micros: sequence as u64,
            sequence,
            reconcile,
        };
        let mut orderer = SnapshotOrderer::<4>::new();
        assert_eq!(orderer.push(event(10, false)), Ok(SequenceStatus::First));
        assert_eq!(
            orderer.push(event(10, false)),
            Ok(SequenceStatus::Duplicate)
        );
        assert_eq!(orderer.push(event(12, false)), Ok(SequenceStatus::Gap));
        assert_eq!(orderer.push(event(40, true)), Ok(SequenceStatus::First));
        assert_eq!(orderer.pop_oldest(), Some(event(40, true)));
        assert_eq!(orderer.pop_oldest(), None);
    }

    #[test]
    fn selects_the_smallest_clean_window_over_ten_thousand_events() {
        const EVENT_COUNT: usize = 10_000;
        let mut clean = [false; 8];

        for window_ms in 1..=8_u64 {
            let mut arrivals = Vec::with_capacity(EVENT_COUNT);
            let mut sequences = [0_u16; 2];
            let mut states = [0_u64; 2];
            for id in 0..EVENT_COUNT {
                let half = if id & 1 == 0 { Half::Left } else { Half::Right };
                let source = half as usize;
                states[source] ^= 1;
                let source_micros = id as u64 * 1_000;
                let transport_delay = if half == Half::Right { 4_000 } else { 0 };
                arrivals.push((
                    source_micros + transport_delay,
                    TimedSnapshot {
                        half,
                        pressed: states[source],
                        source_micros,
                        sequence: sequences[source],
                        reconcile: false,
                    },
                ));
                sequences[source] = sequences[source].wrapping_add(1);
            }
            arrivals.sort_by_key(|(arrival, event)| (*arrival, event.source_micros));

            let mut orderer = SnapshotOrderer::<16>::new();
            let mut merger = SnapshotMerger::default();
            let mut output_count = 0;
            let mut reordered = 0;
            let mut previous_time = None;
            let mut merged = 0;
            for (arrival, event) in arrivals {
                assert!(orderer.push(event).is_ok());
                while let Some(ready) = orderer.pop_ready(arrival, window_ms * 1_000) {
                    if previous_time.is_some_and(|previous| previous >= ready.source_micros) {
                        reordered += 1;
                    }
                    previous_time = Some(ready.source_micros);
                    merged = merger.update(ready.half, ready.pressed);
                    output_count += 1;
                }
            }
            while let Some(ready) = orderer.pop_oldest() {
                if previous_time.is_some_and(|previous| previous >= ready.source_micros) {
                    reordered += 1;
                }
                previous_time = Some(ready.source_micros);
                merged = merger.update(ready.half, ready.pressed);
                output_count += 1;
            }

            let lost = EVENT_COUNT - output_count;
            let duplicate = output_count.saturating_sub(EVENT_COUNT);
            let stuck = merged;
            clean[window_ms as usize - 1] =
                lost == 0 && duplicate == 0 && reordered == 0 && stuck == 0;
        }

        assert_eq!(clean, [false, false, true, true, true, true, true, true]);
        assert_eq!(REORDER_WINDOW_MS, 5);
    }
}
