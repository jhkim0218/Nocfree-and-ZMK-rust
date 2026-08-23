pub const DEEP_SLEEP_SECS: u64 = 5 * 60;
pub const DEEP_SLEEP_PREP_MS: u64 = 300;

pub const fn should_system_off(usb_power: bool, pairing: bool, wake_pin_high: bool) -> bool {
    !usb_power && !pairing && wake_pin_high
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_off_requires_battery_idle_and_an_armed_key_wake_line() {
        assert_eq!(DEEP_SLEEP_SECS, 300);
        assert_eq!(DEEP_SLEEP_PREP_MS, 300);
        assert!(should_system_off(false, false, true));
        assert!(!should_system_off(true, false, true));
        assert!(!should_system_off(false, true, true));
        assert!(!should_system_off(false, false, false));
    }
}
