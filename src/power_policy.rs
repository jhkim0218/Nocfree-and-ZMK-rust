pub const DEEP_SLEEP_SECS: u64 = 5 * 60;
pub const DEEP_SLEEP_PREP_MS: u64 = 300;

pub const fn should_system_off(
    usb_power: bool,
    pairing: bool,
    wake_pin_high: bool,
    ble_host_active: bool,
    keep_ble_awake: bool,
) -> bool {
    !usb_power && !pairing && wake_pin_high && !(keep_ble_awake && ble_host_active)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_off_requires_battery_idle_and_an_armed_key_wake_line() {
        assert_eq!(DEEP_SLEEP_SECS, 300);
        assert_eq!(DEEP_SLEEP_PREP_MS, 300);
        assert!(should_system_off(false, false, true, true, false));
        assert!(!should_system_off(true, false, true, false, false));
        assert!(!should_system_off(false, true, true, false, false));
        assert!(!should_system_off(false, false, false, false, false));
    }

    #[test]
    fn diagnostic_keeps_only_the_ble_host_path_out_of_system_off() {
        assert!(!should_system_off(false, false, true, true, true));
        assert!(should_system_off(false, false, true, false, true));
    }
}
