use core::fmt::{self, Write};
use core::mem;
use core::panic::PanicInfo;

use embassy_futures::join::join;
use embassy_nrf::pac;
use embassy_nrf::pac::gpio::vals::Sense;
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{
    CdcAcmClass, ControlChanged, Receiver as CdcReceiver, Sender as CdcSender,
};
use embassy_usb::driver::Driver as UsbDriver;
use nrf_softdevice::{SocEvent, raw};

use crate::split_diagnostics::{SplitDiagnosticLog, SplitDiagnosticRole, SplitDiagnostics};

#[panic_handler]
fn panic_to_bootloader(_info: &PanicInfo) -> ! {
    unsafe {
        let _ = raw::sd_power_gpregret_clr(0, 0xff);
        let _ = raw::sd_power_gpregret_set(0, 0x57);
    }
    cortex_m::peripheral::SCB::sys_reset()
}

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

pub fn usb_power_detected() -> bool {
    let mut status = 0_u32;
    unsafe {
        assert_eq!(
            raw::sd_power_usbregstatus_get(&mut status),
            raw::NRF_SUCCESS
        );
    }
    status & 1 != 0
}

pub fn key_wake_ready(pin: usize) -> bool {
    pac::P0.in_().read().pin(pin)
}

pub fn try_system_off(pin: usize) -> bool {
    if !key_wake_ready(pin) {
        return false;
    }
    pac::P0
        .pin_cnf(pin)
        .modify(|config| config.set_sense(Sense::LOW));
    let result = unsafe { raw::sd_power_system_off() };
    assert_eq!(result, raw::NRF_SUCCESS);
    loop {
        cortex_m::asm::wfe();
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

struct CdcPacket {
    bytes: [u8; 64],
    len: usize,
}

impl CdcPacket {
    fn new() -> Self {
        Self {
            bytes: [0; 64],
            len: 0,
        }
    }

    fn as_bytes(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

impl Write for CdcPacket {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.len.checked_add(text.len()).ok_or(fmt::Error)?;
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(text.as_bytes());
        self.len = end;
        Ok(())
    }
}

async fn cdc_recovery_monitor<'d, D: UsbDriver<'d>>(receiver: CdcReceiver<'d, D>) -> ! {
    loop {
        if receiver.line_coding().data_rate() == 1200 {
            Timer::after(Duration::from_millis(100)).await;
            if receiver.line_coding().data_rate() == 1200 {
                reboot_to_bootloader();
            }
        }
        Timer::after(Duration::from_millis(20)).await;
    }
}

async fn write_diagnostics<'d, D: UsbDriver<'d>, const N: usize>(
    sender: &mut CdcSender<'d, D>,
    role: SplitDiagnosticRole,
    log: &SplitDiagnosticLog<N>,
) -> bool {
    let mut packet = CdcPacket::new();
    if writeln!(
        packet,
        "NFDIAG1,{},{},{}",
        role as u8 as char,
        log.len(),
        log.dropped()
    )
    .is_err()
        || sender.write_packet(packet.as_bytes()).await.is_err()
    {
        return false;
    }

    for record in log.iter() {
        let mut packet = CdcPacket::new();
        if writeln!(
            packet,
            "{},{},{},{},{:016X}",
            record.timestamp_ms, record.event as u8, record.arg, record.value, record.data
        )
        .is_err()
            || sender.write_packet(packet.as_bytes()).await.is_err()
        {
            return false;
        }
    }
    true
}

async fn cdc_diagnostic_output<'d, D: UsbDriver<'d>, const N: usize>(
    mut sender: CdcSender<'d, D>,
    control: ControlChanged<'d>,
    diagnostics: &'static SplitDiagnostics<N>,
    role: SplitDiagnosticRole,
) -> ! {
    let mut sent_for_open = false;
    loop {
        control.control_changed().await;
        if !sender.dtr() {
            sent_for_open = false;
            continue;
        }
        if sender.line_coding().data_rate() == 115_200 && !sent_for_open {
            let snapshot = diagnostics.snapshot();
            sent_for_open = write_diagnostics(&mut sender, role, &snapshot).await;
        }
    }
}

pub async fn cdc_recovery<'d, D: UsbDriver<'d>, const N: usize>(
    mut cdc: CdcAcmClass<'d, D>,
    diagnostics: &'static SplitDiagnostics<N>,
    role: SplitDiagnosticRole,
) -> ! {
    cdc.wait_connection().await;
    let (sender, receiver, control) = cdc.split_with_control();
    join(
        cdc_recovery_monitor(receiver),
        cdc_diagnostic_output(sender, control, diagnostics, role),
    )
    .await;
    loop {
        cortex_m::asm::wfe();
    }
}
