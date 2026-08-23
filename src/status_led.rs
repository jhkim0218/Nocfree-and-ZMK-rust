pub const UNKNOWN_BATTERY_PERCENT: u8 = u8::MAX;
pub const LOW_BATTERY_PERCENT: u8 = 10;

pub const fn blink_on(tick: u32) -> bool {
    tick % 4 < 2
}

pub const fn low_battery_led_on(percent: u8, tick: u32) -> bool {
    percent != UNKNOWN_BATTERY_PERCENT && percent <= LOW_BATTERY_PERCENT && blink_on(tick)
}

pub const fn pairing_led_on(pairing: bool, tick: u32) -> bool {
    pairing && blink_on(tick)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blink_is_half_a_second_on_and_half_a_second_off() {
        assert_eq!(
            (0..8).map(blink_on).collect::<Vec<_>>(),
            [true, true, false, false, true, true, false, false]
        );
    }

    #[test]
    fn low_battery_blinks_but_unknown_and_healthy_levels_do_not() {
        assert!(low_battery_led_on(10, 0));
        assert!(!low_battery_led_on(10, 2));
        assert!(!low_battery_led_on(11, 0));
        assert!(!low_battery_led_on(UNKNOWN_BATTERY_PERCENT, 0));
    }

    #[test]
    fn blue_led_only_blinks_while_pairing() {
        assert!(pairing_led_on(true, 0));
        assert!(!pairing_led_on(true, 2));
        assert!(!pairing_led_on(false, 0));
    }
}
