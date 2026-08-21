#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OutputMode {
    Auto = 0,
    Usb = 1,
    Ble = 2,
    Disabled = 3,
}

pub const fn routes(mode: OutputMode, usb_connected: bool) -> (bool, bool) {
    match mode {
        OutputMode::Usb => (true, false),
        OutputMode::Ble => (false, true),
        OutputMode::Disabled => (false, false),
        OutputMode::Auto if usb_connected => (true, false),
        OutputMode::Auto => (false, true),
    }
}

pub const fn physical_switch_mode(ble_low: bool, receiver_low: bool) -> Option<OutputMode> {
    match (ble_low, receiver_low) {
        (false, false) => Some(OutputMode::Usb),
        (true, false) => Some(OutputMode::Ble),
        (false, true) => Some(OutputMode::Disabled),
        (true, true) => None,
    }
}

pub const fn releases(previous: (bool, bool), current: (bool, bool)) -> (bool, bool) {
    (previous.0 && !current.0, previous.1 && !current.1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_output_follows_usb_connection() {
        assert_eq!(routes(OutputMode::Auto, false), (false, true));
        assert_eq!(routes(OutputMode::Auto, true), (true, false));
    }

    #[test]
    fn explicit_output_ignores_usb_connection() {
        assert_eq!(routes(OutputMode::Usb, false), (true, false));
        assert_eq!(routes(OutputMode::Ble, true), (false, true));
        assert_eq!(routes(OutputMode::Disabled, true), (false, false));
    }

    #[test]
    fn physical_switch_selects_wired_ble_and_safe_receiver_placeholder() {
        assert_eq!(physical_switch_mode(false, false), Some(OutputMode::Usb));
        assert_eq!(physical_switch_mode(true, false), Some(OutputMode::Ble));
        assert_eq!(
            physical_switch_mode(false, true),
            Some(OutputMode::Disabled)
        );
        assert_eq!(physical_switch_mode(true, true), None);
    }

    #[test]
    fn switching_releases_only_the_previous_output() {
        assert_eq!(releases((true, false), (false, true)), (true, false));
        assert_eq!(releases((false, true), (true, false)), (false, true));
        assert_eq!(releases((true, false), (true, false)), (false, false));
    }
}
