use crate::report::KeyboardReport;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryLevels {
    pub left: u8,
    pub right: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StatusText {
    bytes: [u8; 11],
    len: usize,
}

impl StatusText {
    pub fn new(levels: BatteryLevels) -> Self {
        let mut text = Self {
            bytes: [0; 11],
            len: 0,
        };
        text.push(b'L');
        text.push(b' ');
        text.push_number(levels.left.min(100));
        text.push(b' ');
        text.push(b'R');
        text.push(b' ');
        text.push_number(levels.right.min(100));
        text
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }

    fn push(&mut self, byte: u8) {
        self.bytes[self.len] = byte;
        self.len += 1;
    }

    fn push_number(&mut self, value: u8) {
        if value >= 100 {
            self.push(b'1');
            self.push(b'0');
            self.push(b'0');
        } else {
            if value >= 10 {
                self.push(b'0' + value / 10);
            }
            self.push(b'0' + value % 10);
        }
    }
}

pub fn key_report(byte: u8) -> KeyboardReport {
    let (usage, modifiers) = match byte {
        b'L' => (0x0f, 1 << 1),
        b'R' => (0x15, 1 << 1),
        b'1'..=b'9' => (0x1e + byte - b'1', 0),
        b'0' => (0x27, 0),
        b' ' => (0x2c, 0),
        _ => return KeyboardReport::default(),
    };
    usage_report(usage, modifiers)
}

pub fn usage_report(usage: u8, modifiers: u8) -> KeyboardReport {
    if !(0x04..=0x73).contains(&usage) {
        return KeyboardReport::default();
    }
    let mut report = KeyboardReport {
        modifiers,
        ..KeyboardReport::default()
    };
    let index = (usage - 0x04) as usize;
    report.keys[index / 8] |= 1 << (index & 7);
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_text_matches_requested_format_without_leading_zeroes() {
        assert_eq!(
            StatusText::new(BatteryLevels { left: 7, right: 63 }).as_bytes(),
            b"L 7 R 63"
        );
        assert_eq!(
            StatusText::new(BatteryLevels {
                left: 100,
                right: 0
            })
            .as_bytes(),
            b"L 100 R 0"
        );
    }

    #[test]
    fn status_characters_map_to_keyboard_reports() {
        assert_ne!(key_report(b'L'), KeyboardReport::default());
        assert_ne!(key_report(b'0'), KeyboardReport::default());
        assert_ne!(key_report(b' '), KeyboardReport::default());
        assert_eq!(key_report(b'?'), KeyboardReport::default());
        assert_ne!(usage_report(0x11, 0), KeyboardReport::default());
        assert_eq!(usage_report(0x74, 0), KeyboardReport::default());
    }
}
