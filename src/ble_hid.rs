use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{self, RegisterError, WriteOp};
use nrf_softdevice::ble::{Connection, SecurityMode, Uuid};
use nrf_softdevice::raw;

use crate::battery_ble::BatteryService;
use crate::battery_telemetry::BatterySnapshot;
use crate::dongle_ble::DongleService;
use crate::report::{KeyboardReport, PointerReport};
use crate::usb_descriptor::{
    BLE_HID_REPORT_DESCRIPTOR, KEYBOARD_REPORT_BYTES, POINTER_REPORT_BYTES,
};

const HID_SERVICE: Uuid = Uuid::new_16(0x1812);
const HID_INFORMATION: Uuid = Uuid::new_16(0x2a4a);
const REPORT_MAP: Uuid = Uuid::new_16(0x2a4b);
const HID_CONTROL_POINT: Uuid = Uuid::new_16(0x2a4c);
const HID_REPORT: Uuid = Uuid::new_16(0x2a4d);
const PROTOCOL_MODE: Uuid = Uuid::new_16(0x2a4e);
const REPORT_REFERENCE: Uuid = Uuid::new_16(0x2908);
const KEYBOARD_REPORT_ID: u8 = 1;
const CONSUMER_REPORT_ID: u8 = 2;
const POINTER_REPORT_ID: u8 = 3;

pub struct HidService {
    keyboard_input: u16,
    consumer_input: u16,
    pointer_input: u16,
    input_cccds: [u16; 3],
}

impl HidService {
    fn new(softdevice: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service = ServiceBuilder::new(softdevice, HID_SERVICE)?;

        service
            .add_characteristic(
                HID_INFORMATION,
                Attribute::new([0x11_u8, 0x01, 0x00, 0x01]),
                Metadata::new(Properties::new().read()),
            )?
            .build();
        service
            .add_characteristic(
                REPORT_MAP,
                Attribute::new(BLE_HID_REPORT_DESCRIPTOR).security(SecurityMode::JustWorks),
                Metadata::new(Properties::new().read()),
            )?
            .build();
        service
            .add_characteristic(
                HID_CONTROL_POINT,
                Attribute::new([0_u8]).security(SecurityMode::JustWorks),
                Metadata::new(Properties::new().write_without_response()),
            )?
            .build();
        service
            .add_characteristic(
                PROTOCOL_MODE,
                Attribute::new([1_u8]).security(SecurityMode::JustWorks),
                Metadata::new(Properties::new().read().write_without_response()),
            )?
            .build();

        let mut keyboard_input = service.add_characteristic(
            HID_REPORT,
            Attribute::new([0_u8; KEYBOARD_REPORT_BYTES]).security(SecurityMode::JustWorks),
            Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
        )?;
        keyboard_input.add_descriptor(
            REPORT_REFERENCE,
            Attribute::new([KEYBOARD_REPORT_ID, 1_u8]).security(SecurityMode::JustWorks),
        )?;
        let keyboard_input = keyboard_input.build();

        let mut keyboard_output = service.add_characteristic(
            HID_REPORT,
            Attribute::new([0_u8]).security(SecurityMode::JustWorks),
            Metadata::new(Properties::new().read().write().write_without_response())
                .security(SecurityMode::JustWorks),
        )?;
        keyboard_output.add_descriptor(
            REPORT_REFERENCE,
            Attribute::new([KEYBOARD_REPORT_ID, 2_u8]).security(SecurityMode::JustWorks),
        )?;
        keyboard_output.build();

        let mut consumer_input = service.add_characteristic(
            HID_REPORT,
            Attribute::new([0_u8; 2]).security(SecurityMode::JustWorks),
            Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
        )?;
        consumer_input.add_descriptor(
            REPORT_REFERENCE,
            Attribute::new([CONSUMER_REPORT_ID, 1_u8]).security(SecurityMode::JustWorks),
        )?;
        let consumer_input = consumer_input.build();

        let mut pointer_input = service.add_characteristic(
            HID_REPORT,
            Attribute::new([0_u8; POINTER_REPORT_BYTES]).security(SecurityMode::JustWorks),
            Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
        )?;
        pointer_input.add_descriptor(
            REPORT_REFERENCE,
            Attribute::new([POINTER_REPORT_ID, 1_u8]).security(SecurityMode::JustWorks),
        )?;
        let pointer_input = pointer_input.build();

        service.build();
        Ok(Self {
            keyboard_input: keyboard_input.value_handle,
            consumer_input: consumer_input.value_handle,
            pointer_input: pointer_input.value_handle,
            input_cccds: [
                keyboard_input.cccd_handle,
                consumer_input.cccd_handle,
                pointer_input.cccd_handle,
            ],
        })
    }

    pub fn notification_mask(&self, connection: &Connection) -> u8 {
        let Some(connection_handle) = connection.handle() else {
            return 0;
        };
        let mut mask = 0;
        for (index, handle) in self.input_cccds.iter().enumerate() {
            let mut bytes = [0; 2];
            let mut value = raw::ble_gatts_value_t {
                p_value: bytes.as_mut_ptr(),
                len: 2,
                offset: 0,
            };
            // CCCDs belong to the connection, including subscriptions restored from a bond.
            let result =
                unsafe { raw::sd_ble_gatts_value_get(connection_handle, *handle, &mut value) };
            if result == raw::NRF_SUCCESS && value.len == 2 && bytes == [1, 0] {
                mask |= 1 << index;
            }
        }
        mask
    }

    pub fn notify_keyboard(
        &self,
        connection: &Connection,
        report: &KeyboardReport,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(connection, self.keyboard_input, report.as_bytes())
    }

    pub fn notify_consumer(
        &self,
        connection: &Connection,
        usage: u16,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(connection, self.consumer_input, &usage.to_le_bytes())
    }

    pub fn notify_pointer(
        &self,
        connection: &Connection,
        report: &PointerReport,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(connection, self.pointer_input, report.as_bytes())
    }
}

pub struct BleHidServer {
    pub hid: HidService,
    pub dongle: DongleService,
    pub battery: BatteryService,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BleHidServerEvent {
    DongleSubscribed,
    BatterySubscribed,
    Other,
}

impl BleHidServer {
    pub fn new(
        softdevice: &mut Softdevice,
        snapshot: fn() -> BatterySnapshot,
        level: fn() -> u8,
    ) -> Result<Self, RegisterError> {
        Ok(Self {
            hid: HidService::new(softdevice)?,
            dongle: DongleService::new(softdevice)?,
            battery: BatteryService::new(softdevice, snapshot, level)?,
        })
    }
}

impl gatt_server::Server for BleHidServer {
    type Event = BleHidServerEvent;

    fn on_write(
        &self,
        _connection: &Connection,
        handle: u16,
        _operation: WriteOp,
        _offset: usize,
        data: &[u8],
    ) -> Option<Self::Event> {
        Some(if self.dongle.notifications_enabled(handle, data) {
            BleHidServerEvent::DongleSubscribed
        } else if self.battery.subscription_changed(handle, data) {
            BleHidServerEvent::BatterySubscribed
        } else {
            BleHidServerEvent::Other
        })
    }

    fn on_deferred_read(
        &self,
        handle: u16,
        offset: usize,
        reply: nrf_softdevice::ble::DeferredReadReply,
    ) -> Option<Self::Event> {
        self.battery.on_deferred_read(handle, offset, reply);
        None
    }
}
