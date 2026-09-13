pub const SNAPSHOT_BYTES: usize = 20;
pub const USB_REPORT_BYTES: usize = 21;
pub const UNKNOWN: u8 = 255;
pub const STALE_SECS: u16 = 180;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatterySnapshot {
    pub flags: u8,
    pub keyboard_id: u64,
    pub sequence: u16,
    pub left_percent: u8,
    pub right_percent: u8,
    pub left_age_s: u16,
    pub right_age_s: u16,
    pub selected_output: u8,
}

impl BatterySnapshot {
    pub const fn empty() -> Self {
        Self {
            flags: 0,
            keyboard_id: 0,
            sequence: 0,
            left_percent: UNKNOWN,
            right_percent: UNKNOWN,
            left_age_s: u16::MAX,
            right_age_s: u16::MAX,
            selected_output: UNKNOWN,
        }
    }

    pub fn encode(&self) -> [u8; SNAPSHOT_BYTES] {
        let mut bytes = [0; SNAPSHOT_BYTES];
        bytes[0] = 1;
        bytes[1] = self.flags;
        bytes[2..10].copy_from_slice(&self.keyboard_id.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.sequence.to_le_bytes());
        bytes[12] = self.left_percent;
        bytes[13] = self.right_percent;
        bytes[14..16].copy_from_slice(&self.left_age_s.to_le_bytes());
        bytes[16..18].copy_from_slice(&self.right_age_s.to_le_bytes());
        bytes[18] = self.selected_output;
        bytes
    }

    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SNAPSHOT_BYTES
            || bytes[0] != 1
            || bytes[1] & !0x0f != 0
            || bytes[19] != 0
            || !matches!(bytes[18], 0..=2 | UNKNOWN)
        {
            return None;
        }
        let snapshot = Self {
            flags: bytes[1],
            keyboard_id: u64::from_le_bytes(bytes[2..10].try_into().ok()?),
            sequence: u16::from_le_bytes(bytes[10..12].try_into().ok()?),
            left_percent: bytes[12],
            right_percent: bytes[13],
            left_age_s: u16::from_le_bytes(bytes[14..16].try_into().ok()?),
            right_age_s: u16::from_le_bytes(bytes[16..18].try_into().ok()?),
            selected_output: bytes[18],
        };
        if snapshot.keyboard_id == 0
            && (snapshot.flags != 0
                || snapshot.selected_output != UNKNOWN
                || snapshot.left_age_s != u16::MAX
                || snapshot.right_age_s != u16::MAX)
        {
            return None;
        }
        for (connected, valid, percent, age) in [
            (1, 4, snapshot.left_percent, snapshot.left_age_s),
            (2, 8, snapshot.right_percent, snapshot.right_age_s),
        ] {
            if snapshot.flags & valid != 0 {
                if snapshot.flags & connected == 0 || percent > 100 || age > STALE_SECS {
                    return None;
                }
            } else if percent != UNKNOWN {
                return None;
            }
        }
        Some(snapshot)
    }

    /// Advance original measurement ages, never the USB polling timestamp.
    pub fn aged(mut self, elapsed_secs: u64) -> Self {
        for (connected, valid, percent, age) in [
            (1, 4, &mut self.left_percent, &mut self.left_age_s),
            (2, 8, &mut self.right_percent, &mut self.right_age_s),
        ] {
            if *age != u16::MAX {
                *age = u64::from(*age).saturating_add(elapsed_secs).min(65534) as u16;
            }
            if self.flags & connected == 0
                || self.flags & valid == 0
                || *percent > 100
                || *age > STALE_SECS
            {
                self.flags &= !valid;
                *percent = UNKNOWN;
            }
        }
        self
    }

    pub fn representative(&self) -> Option<u8> {
        let current = self.aged(0);
        [current.left_percent, current.right_percent]
            .into_iter()
            .filter(|value| *value <= 100)
            .min()
    }

    pub fn usb_report(&self, report_id: u8, buffer: &mut [u8]) -> Option<usize> {
        if report_id != 1 || buffer.len() < USB_REPORT_BYTES {
            return None;
        }
        buffer[0] = 1;
        buffer[1..USB_REPORT_BYTES].copy_from_slice(&self.encode());
        Some(USB_REPORT_BYTES)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measured() -> BatterySnapshot {
        BatterySnapshot {
            flags: 15,
            keyboard_id: 0x0807060504030201,
            sequence: 0xfffe,
            left_percent: 0,
            right_percent: 65,
            left_age_s: 179,
            right_age_s: 24,
            selected_output: 2,
        }
    }

    #[test]
    fn layout_preserves_identity_zero_percent_and_unknown() {
        let snapshot = measured();
        let bytes = [
            1, 15, 1, 2, 3, 4, 5, 6, 7, 8, 254, 255, 0, 65, 179, 0, 24, 0, 2, 0,
        ];
        assert_eq!(snapshot.encode(), bytes);
        assert_eq!(BatterySnapshot::decode(&bytes), Some(snapshot));
        assert_eq!(snapshot.representative(), Some(0));
        assert_eq!(
            BatterySnapshot::decode(&BatterySnapshot::empty().encode()),
            Some(BatterySnapshot::empty())
        );
        assert_eq!(BatterySnapshot::empty().representative(), None);
    }

    #[test]
    fn malformed_or_contradictory_values_cannot_replace_a_good_measurement() {
        let good = measured().encode();
        for (index, value) in [
            (0, 2),
            (1, 31),
            (19, 1),
            (18, 3),
            (12, 101),
            (13, 254),
            (12, 255),
            (14, 181),
            (15, 255),
            (1, 14),
            (1, 11),
        ] {
            let mut bad = good;
            bad[index] = value;
            assert_eq!(
                BatterySnapshot::decode(&bad),
                None,
                "index {index}, value {value}"
            );
        }
        assert_eq!(BatterySnapshot::decode(&good[..19]), None);
        assert_eq!(BatterySnapshot::decode(&[0; 21]), None);
    }

    #[test]
    fn polling_never_refreshes_measurements_and_stale_boundary_is_inclusive() {
        let at_boundary = measured().aged(1);
        assert_eq!(at_boundary.left_percent, 0);
        let stale = at_boundary.aged(1);
        assert_eq!(stale.left_percent, UNKNOWN);
        assert_eq!(stale.flags, 11);
        assert_eq!(stale.sequence, measured().sequence);
        assert_eq!(
            BatterySnapshot::decode(&stale.encode())
                .unwrap()
                .aged(30)
                .left_age_s,
            211
        );
        assert_eq!(stale.aged(u64::MAX).left_age_s, 65534);
        assert_eq!(BatterySnapshot::empty().aged(u64::MAX).left_age_s, u16::MAX);
        let mut disconnected = measured();
        disconnected.flags &= !2;
        let disconnected = disconnected.aged(0);
        assert_eq!(disconnected.right_percent, UNKNOWN);
        assert_eq!(disconnected.flags, 5);
    }

    #[test]
    fn usb_feature_contains_id_and_exact_payload_without_partial_writes() {
        let mut buffer = [0; USB_REPORT_BYTES];
        assert_eq!(measured().usb_report(1, &mut buffer), Some(21));
        assert_eq!(buffer[0], 1);
        assert_eq!(&buffer[1..], &measured().encode());
        assert_eq!(measured().usb_report(2, &mut buffer), None);
        let mut short = [42; 20];
        assert_eq!(measured().usb_report(1, &mut short), None);
        assert_eq!(short, [42; 20]);
    }
}
