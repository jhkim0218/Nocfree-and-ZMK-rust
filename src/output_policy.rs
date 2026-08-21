#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum OutputMode {
    Auto = 0,
    Usb = 1,
    Ble = 2,
}

pub const fn routes(mode: OutputMode, usb_connected: bool) -> (bool, bool) {
    match mode {
        OutputMode::Usb => (true, false),
        OutputMode::Ble => (false, true),
        OutputMode::Auto if usb_connected => (true, false),
        OutputMode::Auto => (false, true),
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
    }

    #[test]
    fn switching_releases_only_the_previous_output() {
        assert_eq!(releases((true, false), (false, true)), (true, false));
        assert_eq!(releases((false, true), (true, false)), (false, true));
        assert_eq!(releases((true, false), (true, false)), (false, false));
    }
}
