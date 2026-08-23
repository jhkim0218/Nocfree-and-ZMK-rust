#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BacklightCommand {
    Toggle,
    Down,
    Up,
    Idle,
    Wake,
}

pub const AUTO_OFF_SECS: u64 = 30;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BacklightState {
    pub enabled: bool,
    pub percent: u8,
    timed_out: bool,
}

impl Default for BacklightState {
    fn default() -> Self {
        Self {
            enabled: true,
            percent: 20,
            timed_out: false,
        }
    }
}

impl BacklightState {
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
            (max as u32 * self.percent as u32 / 100) as u16
        } else {
            0
        };
        // SimplePwm emits inverted nRF polarity; ZMK verified normal polarity on P0.20.
        max - active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backlight_uses_verified_twenty_percent_steps() {
        let mut state = BacklightState::default();
        assert_eq!(state.duty(1_000), 800);
        state.apply(BacklightCommand::Down);
        assert_eq!(state.duty(1_000), 1_000);
        state.apply(BacklightCommand::Toggle);
        assert_eq!(state.duty(1_000), 1_000);
        state.apply(BacklightCommand::Up);
        assert_eq!(state.duty(1_000), 1_000);
        state.apply(BacklightCommand::Toggle);
        assert_eq!(state.duty(1_000), 800);
        for _ in 0..10 {
            state.apply(BacklightCommand::Up);
        }
        assert_eq!(state.duty(1_000), 0);
    }

    #[test]
    fn auto_off_wakes_without_overriding_the_manual_setting() {
        assert_eq!(AUTO_OFF_SECS, 30);
        let mut state = BacklightState::default();
        state.apply(BacklightCommand::Idle);
        assert_eq!(state.duty(1_000), 1_000);
        state.apply(BacklightCommand::Wake);
        assert_eq!(state.duty(1_000), 800);

        state.apply(BacklightCommand::Toggle);
        state.apply(BacklightCommand::Idle);
        state.apply(BacklightCommand::Wake);
        assert_eq!(state.duty(1_000), 1_000);
    }
}
