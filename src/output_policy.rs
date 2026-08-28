#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OutputMode {
    Auto = 0,
    Usb = 1,
    Ble = 2,
    Disabled = 3,
    Dongle = 4,
}

pub const fn routes(mode: OutputMode, usb_connected: bool) -> (bool, bool, bool) {
    match mode {
        OutputMode::Usb => (true, false, false),
        OutputMode::Ble => (false, true, false),
        OutputMode::Dongle => (false, false, true),
        OutputMode::Disabled => (false, false, false),
        OutputMode::Auto if usb_connected => (true, false, false),
        OutputMode::Auto => (false, true, false),
    }
}

pub const fn physical_switch_mode(ble_low: bool, receiver_low: bool) -> Option<OutputMode> {
    match (ble_low, receiver_low) {
        (false, false) => Some(OutputMode::Usb),
        (true, false) => Some(OutputMode::Ble),
        (false, true) => Some(OutputMode::Dongle),
        (true, true) => None,
    }
}

pub const fn releases(
    previous: (bool, bool, bool),
    current: (bool, bool, bool),
) -> (bool, bool, bool) {
    (
        previous.0 && !current.0,
        previous.1 && !current.1,
        previous.2 && !current.2,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn automatic_output_follows_usb_connection() {
        assert_eq!(routes(OutputMode::Auto, false), (false, true, false));
        assert_eq!(routes(OutputMode::Auto, true), (true, false, false));
    }

    #[test]
    fn explicit_output_ignores_usb_connection() {
        assert_eq!(routes(OutputMode::Usb, false), (true, false, false));
        assert_eq!(routes(OutputMode::Ble, true), (false, true, false));
        assert_eq!(routes(OutputMode::Dongle, true), (false, false, true));
        assert_eq!(routes(OutputMode::Disabled, true), (false, false, false));
    }

    #[test]
    fn physical_switch_selects_wired_ble_and_dongle() {
        assert_eq!(physical_switch_mode(false, false), Some(OutputMode::Usb));
        assert_eq!(physical_switch_mode(true, false), Some(OutputMode::Ble));
        assert_eq!(physical_switch_mode(false, true), Some(OutputMode::Dongle));
        assert_eq!(physical_switch_mode(true, true), None);
    }

    #[test]
    fn switching_releases_only_the_previous_output() {
        assert_eq!(
            releases((true, false, false), (false, true, false)),
            (true, false, false)
        );
        assert_eq!(
            releases((false, true, false), (false, false, true)),
            (false, true, false)
        );
        assert_eq!(
            releases((false, false, true), (true, false, false)),
            (false, false, true)
        );
    }
}
