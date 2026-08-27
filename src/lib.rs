#![cfg_attr(not(test), no_std)]

#[cfg(all(feature = "split-softdevice", feature = "standalone-critical-section"))]
compile_error!("select exactly one ARM critical-section implementation");

pub mod backlight;
pub mod battery;
pub mod battery_status;
#[cfg(all(target_arch = "arm", feature = "split-softdevice"))]
pub mod ble_hid;
pub mod bond_record;
#[cfg(all(target_arch = "arm", feature = "split-softdevice"))]
pub mod bond_store;
#[cfg(target_arch = "arm")]
pub mod hardware_scanner;
pub mod keymap;
pub mod link_keymap;
pub mod link_protocol;
#[cfg(all(target_arch = "arm", feature = "split-softdevice"))]
pub mod link_usb;
pub mod output_policy;
#[cfg(target_arch = "arm")]
pub mod output_router;
pub mod pca9555;
#[cfg(all(target_arch = "arm", feature = "split-softdevice"))]
pub mod platform;
pub mod power_policy;
pub mod report;
pub mod scanner;
#[cfg(all(target_arch = "arm", feature = "split-softdevice"))]
pub mod split_ble;
pub mod split_diagnostics;
pub mod split_protocol;
pub mod status_led;
pub mod usb_descriptor;
