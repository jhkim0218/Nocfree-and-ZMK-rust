use core::mem;

use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::CdcAcmClass;
use embassy_usb::driver::Driver as UsbDriver;
use nrf_softdevice::{SocEvent, raw};

pub fn softdevice_config(device_name: &'static [u8]) -> nrf_softdevice::Config {
    nrf_softdevice::Config {
        clock: Some(raw::nrf_clock_lf_cfg_t {
            source: raw::NRF_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: 16,
            rc_temp_ctiv: 2,
            accuracy: raw::NRF_CLOCK_LF_ACCURACY_500_PPM as u8,
        }),
        conn_gap: Some(raw::ble_gap_conn_cfg_t {
            conn_count: 2,
            event_length: 24,
        }),
        conn_gatt: Some(raw::ble_gatt_conn_cfg_t { att_mtu: 64 }),
        gatts_attr_tab_size: Some(raw::ble_gatts_cfg_attr_tab_size_t {
            attr_tab_size: raw::BLE_GATTS_ATTR_TAB_SIZE_DEFAULT,
        }),
        gap_role_count: Some(raw::ble_gap_cfg_role_count_t {
            adv_set_count: 1,
            periph_role_count: 1,
            central_role_count: 1,
            central_sec_count: 1,
            _bitfield_1: raw::ble_gap_cfg_role_count_t::new_bitfield_1(0),
        }),
        gap_device_name: Some(raw::ble_gap_cfg_device_name_t {
            p_value: device_name.as_ptr() as _,
            current_len: device_name.len() as u16,
            max_len: device_name.len() as u16,
            write_perm: unsafe { mem::zeroed() },
            _bitfield_1: raw::ble_gap_cfg_device_name_t::new_bitfield_1(
                raw::BLE_GATTS_VLOC_STACK as u8,
            ),
        }),
        ..Default::default()
    }
}

pub fn enable_usb_power_events() -> (bool, bool) {
    let mut status = 0_u32;
    unsafe {
        assert_eq!(raw::sd_power_usbpwrrdy_enable(1), raw::NRF_SUCCESS);
        assert_eq!(raw::sd_power_usbdetected_enable(1), raw::NRF_SUCCESS);
        assert_eq!(raw::sd_power_usbremoved_enable(1), raw::NRF_SUCCESS);
        assert_eq!(
            raw::sd_power_usbregstatus_get(&mut status),
            raw::NRF_SUCCESS
        );
    }
    (status & 1 != 0, status & 2 != 0)
}

pub fn update_usb_power(vbus: &SoftwareVbusDetect, event: SocEvent) {
    match event {
        SocEvent::PowerUsbDetected => vbus.detected(true),
        SocEvent::PowerUsbRemoved => vbus.detected(false),
        SocEvent::PowerUsbPowerReady => vbus.ready(),
        _ => {}
    }
}

pub fn reboot_to_bootloader() -> ! {
    unsafe {
        assert_eq!(raw::sd_power_gpregret_clr(0, 0xff), raw::NRF_SUCCESS);
        assert_eq!(raw::sd_power_gpregret_set(0, 0x57), raw::NRF_SUCCESS);
    }
    cortex_m::peripheral::SCB::sys_reset()
}

pub fn reboot_application() -> ! {
    cortex_m::peripheral::SCB::sys_reset()
}

pub async fn cdc_recovery<'d, D: UsbDriver<'d>>(mut cdc: CdcAcmClass<'d, D>) -> ! {
    cdc.wait_connection().await;
    loop {
        if cdc.line_coding().data_rate() == 1200 {
            Timer::after(Duration::from_millis(100)).await;
            if cdc.line_coding().data_rate() == 1200 {
                reboot_to_bootloader();
            }
        }
        Timer::after(Duration::from_millis(20)).await;
    }
}
