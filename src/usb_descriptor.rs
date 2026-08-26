use crate::report::{KEY_BITMAP_BITS, LAST_BITMAP_USAGE};

pub use crate::report::KEYBOARD_REPORT_BYTES;
pub const CONSUMER_REPORT_BYTES: usize = 2;

#[rustfmt::skip]
pub const KEYBOARD_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, // Usage Page (Generic Desktop)
    0x09, 0x06, // Usage (Keyboard)
    0xa1, 0x01, // Collection (Application)
    0x05, 0x07, // Usage Page (Keyboard)
    0x19, 0xe0, // Usage Minimum (Left Control)
    0x29, 0xe7, // Usage Maximum (Right GUI)
    0x15, 0x00, // Logical Minimum (0)
    0x25, 0x01, // Logical Maximum (1)
    0x75, 0x01, // Report Size (1)
    0x95, 0x08, // Report Count (8)
    0x81, 0x02, // Input (Data, Variable, Absolute)
    0x75, 0x08, // Report Size (8)
    0x95, 0x01, // Report Count (1)
    0x81, 0x01, // Input (Constant)
    0x05, 0x08, // Usage Page (LEDs)
    0x19, 0x01, // Usage Minimum (Num Lock)
    0x29, 0x05, // Usage Maximum (Kana)
    0x75, 0x01, // Report Size (1)
    0x95, 0x05, // Report Count (5)
    0x91, 0x02, // Output (Data, Variable, Absolute)
    0x75, 0x03, // Report Size (3)
    0x95, 0x01, // Report Count (1)
    0x91, 0x01, // Output (Constant)
    0x05, 0x07, // Usage Page (Keyboard)
    0x19, 0x04, // Usage Minimum (A)
    0x29, LAST_BITMAP_USAGE, // Usage Maximum
    0x15, 0x00, // Logical Minimum (0)
    0x25, 0x01, // Logical Maximum (1)
    0x75, 0x01, // Report Size (1)
    0x95, KEY_BITMAP_BITS, // Report Count
    0x81, 0x02, // Input (Data, Variable, Absolute)
    0xc0, // End Collection
];

pub const CONSUMER_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x0c, // Usage Page (Consumer)
    0x09, 0x01, // Usage (Consumer Control)
    0xa1, 0x01, // Collection (Application)
    0x15, 0x00, // Logical Minimum (0)
    0x26, 0xff, 0x03, // Logical Maximum (0x03ff)
    0x19, 0x00, // Usage Minimum (0)
    0x2a, 0xff, 0x03, // Usage Maximum (0x03ff)
    0x75, 0x10, // Report Size (16)
    0x95, 0x01, // Report Count (1)
    0x81, 0x00, // Input (Data, Array, Absolute)
    0xc0, // End Collection
];

#[rustfmt::skip]
pub const BLE_HID_REPORT_DESCRIPTOR: &[u8] = &[
    0x05, 0x01, 0x09, 0x06, 0xa1, 0x01, 0x85, 0x01, 0x05, 0x07, 0x19, 0xe0, 0x29, 0xe7, 0x15, 0x00,
    0x25, 0x01, 0x75, 0x01, 0x95, 0x08, 0x81, 0x02, 0x75, 0x08, 0x95, 0x01, 0x81, 0x01, 0x05, 0x08,
    0x19, 0x01, 0x29, 0x05, 0x75, 0x01, 0x95, 0x05, 0x91, 0x02, 0x75, 0x03, 0x95, 0x01, 0x91, 0x01,
    0x05, 0x07, 0x19, 0x04, 0x29, LAST_BITMAP_USAGE, 0x15, 0x00, 0x25, 0x01, 0x75, 0x01, 0x95,
    KEY_BITMAP_BITS, 0x81, 0x02, 0xc0, 0x05, 0x0c, 0x09, 0x01, 0xa1, 0x01, 0x85, 0x02, 0x15, 0x00,
    0x26, 0xff, 0x03, 0x19, 0x00, 0x2a, 0xff, 0x03, 0x75, 0x10, 0x95, 0x01, 0x81, 0x00, 0xc0,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_sizes_match_the_descriptors() {
        #[cfg(feature = "layout-jis")]
        assert_eq!(KEYBOARD_REPORT_BYTES, 19);
        #[cfg(not(feature = "layout-jis"))]
        assert_eq!(KEYBOARD_REPORT_BYTES, 16);
        assert!(
            KEYBOARD_REPORT_DESCRIPTOR
                .windows(2)
                .any(|bytes| { bytes == [0x29, LAST_BITMAP_USAGE] })
        );
        assert!(
            KEYBOARD_REPORT_DESCRIPTOR
                .windows(2)
                .any(|bytes| { bytes == [0x95, KEY_BITMAP_BITS] })
        );
        assert_eq!(CONSUMER_REPORT_BYTES, 2);
        assert!(KEYBOARD_REPORT_DESCRIPTOR.ends_with(&[0xc0]));
        assert!(CONSUMER_REPORT_DESCRIPTOR.ends_with(&[0xc0]));
        assert!(
            BLE_HID_REPORT_DESCRIPTOR
                .windows(2)
                .any(|bytes| bytes == [0x85, 0x01])
        );
        assert!(
            BLE_HID_REPORT_DESCRIPTOR
                .windows(2)
                .any(|bytes| bytes == [0x85, 0x02])
        );
    }
}
