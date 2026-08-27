#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BacklightCommand {
    Toggle,
    Down,
    Up,
    Idle,
    Wake,
}

pub const AUTO_OFF_SECS: u64 = 30;
// NocFree &'s backlight input did not reproduce intermediate duty levels at 10 kHz.
// 1 kHz stays above visible flicker while giving the external circuit time to settle.
pub const BACKLIGHT_PWM_HZ: u32 = 1_000;
pub const BACKLIGHT_STATE_VERSION: u8 = 1;
pub const BACKLIGHT_STATE_BYTES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BacklightState {
    pub enabled: bool,
    pub percent: u8,
    timed_out: bool,
}

impl Default for BacklightState {
    fn default() -> Self {
        Self::new()
    }
}

impl BacklightState {
    pub const fn new() -> Self {
        Self {
            enabled: true,
            percent: 20,
            timed_out: false,
        }
    }

    pub fn apply(&mut self, command: BacklightCommand) {
        match command {
            BacklightCommand::Toggle => {
                self.timed_out = false;
                self.enabled = !self.enabled;
            }
            BacklightCommand::Down => {
                self.timed_out = false;
                self.percent = self.percent.saturating_sub(20);
            }
            BacklightCommand::Up => {
                self.timed_out = false;
                self.percent = self.percent.saturating_add(20).min(100);
            }
            BacklightCommand::Idle => self.timed_out = true,
            BacklightCommand::Wake => self.timed_out = false,
        }
    }

    pub const fn duty(self, max: u16) -> u16 {
        let active = if self.enabled && !self.timed_out {
            // Linear remains available for hardware comparison; ANSI defaults to the
            // perceptual curve because its upper levels were less compressed by eye.
            #[cfg(not(feature = "backlight-perceptual"))]
            {
                (max as u32 * self.percent as u32 / 100) as u16
            }
            #[cfg(feature = "backlight-perceptual")]
            {
                let percent = self.percent as u32;
                (max as u32 * percent * percent / 10_000) as u16
            }
        } else {
            0
        };
        // SimplePwm emits inverted nRF polarity; ZMK verified normal polarity on P0.20.
        max - active
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BacklightSnapshot {
    pub state: BacklightState,
    pub generation: u8,
}

impl Default for BacklightSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

impl BacklightSnapshot {
    pub const fn new() -> Self {
        Self {
            state: BacklightState::new(),
            generation: 0,
        }
    }

    pub fn apply(&mut self, command: BacklightCommand) {
        self.state.apply(command);
        self.generation = self.generation.wrapping_add(1);
    }

    pub const fn encode(self) -> [u8; BACKLIGHT_STATE_BYTES] {
        let flags = self.state.enabled as u8 | ((self.state.timed_out as u8) << 1);
        [
            BACKLIGHT_STATE_VERSION,
            flags,
            self.state.percent,
            self.generation,
        ]
    }

    pub const fn decode(bytes: [u8; BACKLIGHT_STATE_BYTES]) -> Option<Self> {
        if bytes[0] != BACKLIGHT_STATE_VERSION || bytes[1] & !0b11 != 0 || bytes[2] > 100 {
            return None;
        }
        Some(Self {
            state: BacklightState {
                enabled: bytes[1] & 1 != 0,
                timed_out: bytes[1] & 2 != 0,
                percent: bytes[2],
            },
            generation: bytes[3],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backlight_uses_selected_curve() {
        let mut state = BacklightState::default();
        assert_eq!(BACKLIGHT_PWM_HZ, 1_000);
        state.apply(BacklightCommand::Down);
        let mut duties = [0; 6];
        for duty in &mut duties {
            *duty = state.duty(1_000);
            state.apply(BacklightCommand::Up);
        }
        #[cfg(not(feature = "backlight-perceptual"))]
        assert_eq!(duties, [1_000, 800, 600, 400, 200, 0]);
        #[cfg(feature = "backlight-perceptual")]
        assert_eq!(duties, [1_000, 960, 840, 640, 360, 0]);
        assert!(duties.windows(2).all(|pair| pair[0] > pair[1]));

        state = BacklightState::default();
        state.apply(BacklightCommand::Down);
        assert_eq!(state.duty(100), 100);
        state.apply(BacklightCommand::Toggle);
        assert_eq!(state.duty(100), 100);
        state.apply(BacklightCommand::Up);
        assert_eq!(state.duty(100), 100);
        state.apply(BacklightCommand::Toggle);
        #[cfg(not(feature = "backlight-perceptual"))]
        assert_eq!(state.duty(100), 80);
        #[cfg(feature = "backlight-perceptual")]
        assert_eq!(state.duty(100), 96);
        for _ in 0..10 {
            state.apply(BacklightCommand::Up);
        }
        assert_eq!(state.duty(100), 0);
    }

    #[test]
    fn auto_off_wakes_without_overriding_the_manual_setting() {
        assert_eq!(AUTO_OFF_SECS, 30);
        let mut state = BacklightState::default();
        state.apply(BacklightCommand::Idle);
        assert_eq!(state.duty(1_000), 1_000);
        state.apply(BacklightCommand::Wake);
        #[cfg(not(feature = "backlight-perceptual"))]
        assert_eq!(state.duty(1_000), 800);
        #[cfg(feature = "backlight-perceptual")]
        assert_eq!(state.duty(1_000), 960);

        state.apply(BacklightCommand::Toggle);
        state.apply(BacklightCommand::Idle);
        state.apply(BacklightCommand::Wake);
        assert_eq!(state.duty(1_000), 1_000);
    }

    #[test]
    fn absolute_snapshot_converges_and_is_idempotent() {
        let mut left = BacklightSnapshot::new();
        left.apply(BacklightCommand::Up);
        left.apply(BacklightCommand::Idle);

        let mut right = BacklightSnapshot::new();
        right.apply(BacklightCommand::Toggle);
        assert_ne!(right.state, left.state);

        let received = BacklightSnapshot::decode(left.encode()).unwrap();
        right = received;
        assert_eq!(right, left);
        right = BacklightSnapshot::decode(left.encode()).unwrap();
        assert_eq!(right, left);
    }

    #[test]
    fn absolute_snapshot_rejects_unknown_or_invalid_wire_values() {
        assert_eq!(BacklightSnapshot::decode([2, 1, 20, 0]), None);
        assert_eq!(BacklightSnapshot::decode([1, 4, 20, 0]), None);
        assert_eq!(BacklightSnapshot::decode([1, 1, 101, 0]), None);
    }

    #[test]
    fn snapshot_generation_wraps_without_changing_state_encoding() {
        let mut snapshot = BacklightSnapshot {
            state: BacklightState::new(),
            generation: u8::MAX,
        };
        snapshot.apply(BacklightCommand::Toggle);
        assert_eq!(snapshot.generation, 0);
        assert_eq!(BacklightSnapshot::decode(snapshot.encode()), Some(snapshot));
    }
}
