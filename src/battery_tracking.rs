use crate::battery_telemetry::{BatterySnapshot, UNKNOWN};

pub const SAMPLE_INTERVAL_SECS: u64 = 60;

#[derive(Clone, Copy)]
struct Sample {
    percent: u8,
    at: Option<u64>,
    discharge_percent: u8,
}

impl Sample {
    const fn empty() -> Self {
        Self {
            percent: UNKNOWN,
            at: None,
            discharge_percent: UNKNOWN,
        }
    }

    fn update(&mut self, percent: u8, now: u64, allow_rise: bool) {
        if percent > 100 {
            return;
        }
        self.percent = percent;
        self.at = Some(now);
        self.discharge_percent = if allow_rise {
            percent
        } else {
            self.discharge_percent.min(percent)
        };
    }

    fn age(self, now: u64) -> u16 {
        self.at
            .map_or(u16::MAX, |at| now.saturating_sub(at).min(65534) as u16)
    }
}

pub struct BatteryTracker {
    id: u64,
    sequence: u16,
    left: Sample,
    right: Sample,
    right_connected: bool,
    output: u8,
    last_bas: u8,
}

impl BatteryTracker {
    pub const fn new(id: u64) -> Self {
        Self {
            id,
            sequence: 0,
            left: Sample::empty(),
            right: Sample::empty(),
            right_connected: false,
            output: 255,
            last_bas: 0,
        }
    }

    pub fn update_left(&mut self, percent: u8, now: u64, external_power: bool) {
        self.left.update(percent, now, external_power);
        self.sequence = self.sequence.wrapping_add(1);
    }

    pub fn update_right(&mut self, percent: u8, now: u64) {
        if self.right_connected {
            self.right.update(percent, now, false);
            self.sequence = self.sequence.wrapping_add(1);
        }
    }

    pub fn set_right_connected(&mut self, connected: bool) {
        if self.right_connected != connected {
            self.right_connected = connected;
            // Preserve the discharge floor over a radio reconnect, but never its freshness.
            self.right.percent = UNKNOWN;
            self.right.at = None;
            self.sequence = self.sequence.wrapping_add(1);
        }
    }

    pub fn snapshot(&mut self, now: u64, output: u8) -> BatterySnapshot {
        if self.output != output {
            self.output = output;
            self.sequence = self.sequence.wrapping_add(1);
        }
        BatterySnapshot {
            flags: 1
                | (u8::from(self.right_connected) * 2)
                | (u8::from(self.left.at.is_some()) * 4)
                | (u8::from(self.right_connected && self.right.at.is_some()) * 8),
            keyboard_id: self.id,
            sequence: self.sequence,
            left_percent: self.left.percent,
            right_percent: self.right.percent,
            left_age_s: self.left.age(now),
            right_age_s: self.right.age(now),
            selected_output: output,
        }
        .aged(0)
    }

    pub fn bas_level(&mut self, now: u64, output: u8) -> u8 {
        let mut snapshot = self.snapshot(now, output);
        if snapshot.flags & 4 != 0 {
            snapshot.left_percent = self.left.discharge_percent;
        }
        if snapshot.flags & 8 != 0 {
            snapshot.right_percent = self.right.discharge_percent;
        }
        if let Some(level) = snapshot.representative() {
            self.last_bas = level;
        }
        self.last_bas
    }
}

pub struct BatteryCache {
    value: Option<(BatterySnapshot, u64)>,
}

impl Default for BatteryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl BatteryCache {
    pub const fn new() -> Self {
        Self { value: None }
    }

    pub fn clear(&mut self) {
        self.value = None;
    }

    pub fn accept(&mut self, bytes: &[u8], now: u64) -> bool {
        let Some(mut next) = BatterySnapshot::decode(bytes) else {
            return false;
        };
        if let Some((previous, at)) = self.value {
            if previous.keyboard_id != next.keyboard_id {
                return false;
            }
            let distance = next.sequence.wrapping_sub(previous.sequence);
            if distance >= 0x8000 {
                return false;
            }
            if distance == 0 {
                let previous = previous.aged(now.saturating_sub(at));
                // Re-reading one sample must not reset its clock or resurrect stale values.
                next.left_age_s = next.left_age_s.max(previous.left_age_s);
                next.right_age_s = next.right_age_s.max(previous.right_age_s);
                next.flags &= previous.flags;
                next.left_percent = previous.left_percent;
                next.right_percent = previous.right_percent;
                next.selected_output = previous.selected_output;
                next = next.aged(0);
            }
        }
        self.value = Some((next, now));
        true
    }

    pub fn snapshot(&self, now: u64) -> BatterySnapshot {
        self.value.map_or(BatterySnapshot::empty(), |(value, at)| {
            value.aged(now.saturating_sub(at))
        })
    }
}

/// The deadline survives unrelated backlight events and explicit measurement requests.
pub struct SampleDeadline(u64);

impl SampleDeadline {
    pub fn new(now: u64) -> Self {
        Self(now + SAMPLE_INTERVAL_SECS)
    }
    pub fn due(&self, now: u64) -> bool {
        now >= self.0
    }
    pub fn next(&self) -> u64 {
        self.0
    }
    pub fn completed(&mut self, now: u64) {
        self.0 = now + SAMPLE_INTERVAL_SECS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reconnect_requires_a_new_sample_and_does_not_reset_discharge_floor() {
        let mut tracker = BatteryTracker::new(42);
        tracker.update_left(80, 0, false);
        tracker.set_right_connected(true);
        tracker.update_right(20, 0);
        assert_eq!(tracker.bas_level(0, 1), 20);
        tracker.set_right_connected(false);
        assert_eq!(tracker.snapshot(1, 1).right_percent, UNKNOWN);
        tracker.set_right_connected(true);
        assert_eq!(tracker.snapshot(2, 1).right_percent, UNKNOWN);
        tracker.update_right(25, 3);
        assert_eq!(tracker.bas_level(3, 1), 20);
        assert_eq!(tracker.snapshot(184, 1).representative(), None);
    }

    #[test]
    fn repeated_reads_and_invalid_packets_cannot_rejuvenate_battery() {
        let mut tracker = BatteryTracker::new(42);
        tracker.update_left(0, 10, false);
        let packet = tracker.snapshot(10, 0).encode();
        let mut cache = BatteryCache::new();
        assert!(cache.accept(&packet, 20));
        assert!(cache.accept(&packet, 120));
        assert_eq!(cache.snapshot(120).left_age_s, 100);
        assert!(!cache.accept(&packet[..19], 130));
        assert_eq!(cache.snapshot(200).left_percent, 0);
        assert_eq!(cache.snapshot(201).left_percent, UNKNOWN);
        assert!(cache.accept(&packet, 202));
        assert_eq!(cache.snapshot(202).left_percent, UNKNOWN);
        cache.clear();
        assert_eq!(cache.snapshot(203).keyboard_id, 0);
    }

    #[test]
    fn receiver_rejects_old_frames_and_other_keyboards_until_disconnect() {
        let mut packet = BatterySnapshot::empty();
        packet.keyboard_id = 42;
        packet.flags = 1;
        packet.selected_output = 2;
        packet.sequence = 65535;
        let mut cache = BatteryCache::new();
        assert!(cache.accept(&packet.encode(), 0));
        packet.sequence = 0;
        assert!(cache.accept(&packet.encode(), 1));
        packet.sequence = 65535;
        assert!(!cache.accept(&packet.encode(), 2));
        packet.keyboard_id = 43;
        packet.sequence = 1;
        assert!(!cache.accept(&packet.encode(), 3));
        cache.clear();
        assert!(cache.accept(&packet.encode(), 3));
    }

    #[test]
    fn busy_backlight_does_not_move_the_measurement_deadline() {
        let mut deadline = SampleDeadline::new(0);
        for second in 0..60 {
            assert!(!deadline.due(second));
        }
        assert!(deadline.due(60));
        assert!(deadline.due(61));
        deadline.completed(61);
        assert_eq!(deadline.next(), 121);
    }
}
