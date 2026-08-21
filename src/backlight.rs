#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BacklightCommand {
    Toggle,
    Down,
    Up,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BacklightState {
    pub enabled: bool,
    pub percent: u8,
}

impl Default for BacklightState {
    fn default() -> Self {
        Self {
            enabled: true,
            percent: 20,
        }
    }
}

impl BacklightState {
    pub fn apply(&mut self, command: BacklightCommand) {
        match command {
            BacklightCommand::Toggle => self.enabled = !self.enabled,
            BacklightCommand::Down => self.percent = self.percent.saturating_sub(20),
            BacklightCommand::Up => self.percent = self.percent.saturating_add(20).min(100),
        }
    }

    pub const fn duty(self, max: u16) -> u16 {
        if self.enabled {
            (max as u32 * self.percent as u32 / 100) as u16
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backlight_uses_verified_twenty_percent_steps() {
        let mut state = BacklightState::default();
        assert_eq!(state.duty(1_000), 200);
        state.apply(BacklightCommand::Down);
        assert_eq!(state.duty(1_000), 0);
        state.apply(BacklightCommand::Toggle);
        assert_eq!(state.duty(1_000), 0);
        state.apply(BacklightCommand::Up);
        assert_eq!(state.duty(1_000), 0);
        state.apply(BacklightCommand::Toggle);
        assert_eq!(state.duty(1_000), 200);
        for _ in 0..10 {
            state.apply(BacklightCommand::Up);
        }
        assert_eq!(state.duty(1_000), 1_000);
    }
}
