use embassy_usb::class::hid::{ReportId, RequestHandler};

use crate::battery_telemetry::BatterySnapshot;

pub const REPORT_DESCRIPTOR: &[u8] = &[
    0x06, 0x60, 0xff, // Usage Page (vendor 0xff60)
    0x09, 0x01, // Usage (battery)
    0xa1, 0x01, // Application collection
    0x85, 0x01, // Report ID 1
    0x15, 0x00, // Logical minimum 0
    0x26, 0xff, 0x00, // Logical maximum 255
    0x75, 0x08, // Report size 8 bits
    0x95, 0x14, // Report count 20
    0x09, 0x01, 0xb1, 0x02, // Feature (Data, Variable, Absolute)
    0x09, 0x01, 0x81, 0x02, // Input (Data, Variable, Absolute)
    0xc0,
];

pub struct BatteryUsbHandler {
    pub snapshot: fn() -> BatterySnapshot,
}

impl RequestHandler for BatteryUsbHandler {
    fn get_report(&mut self, id: ReportId, buffer: &mut [u8]) -> Option<usize> {
        match id {
            // Embassy returns these bytes verbatim; it does not prepend the report ID.
            ReportId::Feature(1) | ReportId::In(1) => (self.snapshot)().usb_report(1, buffer),
            _ => None,
        }
    }
}
