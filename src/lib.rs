#![cfg_attr(not(test), no_std)]

#[cfg(target_arch = "arm")]
pub mod ble_hid;
pub mod bond_record;
#[cfg(target_arch = "arm")]
pub mod bond_store;
#[cfg(target_arch = "arm")]
pub mod hardware_scanner;
pub mod keymap;
pub mod link_keymap;
pub mod link_protocol;
#[cfg(target_arch = "arm")]
pub mod link_usb;
pub mod output_policy;
#[cfg(target_arch = "arm")]
pub mod output_router;
pub mod pca9555;
#[cfg(target_arch = "arm")]
pub mod platform;
pub mod report;
pub mod scanner;
#[cfg(target_arch = "arm")]
pub mod split_ble;
pub mod split_protocol;
pub mod usb_descriptor;
