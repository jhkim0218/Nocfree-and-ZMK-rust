use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{self, RegisterError, WriteOp};
use nrf_softdevice::ble::{Connection, SecurityMode, Uuid};

use crate::report::KeyboardReport;
use crate::usb_descriptor::BLE_HID_REPORT_DESCRIPTOR;

const HID_SERVICE: Uuid = Uuid::new_16(0x1812);
const HID_INFORMATION: Uuid = Uuid::new_16(0x2a4a);
const REPORT_MAP: Uuid = Uuid::new_16(0x2a4b);
const HID_CONTROL_POINT: Uuid = Uuid::new_16(0x2a4c);
const HID_REPORT: Uuid = Uuid::new_16(0x2a4d);
const PROTOCOL_MODE: Uuid = Uuid::new_16(0x2a4e);
const REPORT_REFERENCE: Uuid = Uuid::new_16(0x2908);
const KEYBOARD_REPORT_ID: u8 = 1;
const CONSUMER_REPORT_ID: u8 = 2;

pub struct HidService {
    keyboard_input: u16,
    consumer_input: u16,
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
            Attribute::new([0_u8; 16]).security(SecurityMode::JustWorks),
            Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
        )?;
        keyboard_input.add_descriptor(
            REPORT_REFERENCE,
            Attribute::new([KEYBOARD_REPORT_ID, 1_u8]).security(SecurityMode::JustWorks),
        )?;
        let keyboard_input = keyboard_input.build().value_handle;

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
        let consumer_input = consumer_input.build().value_handle;

        service.build();
        Ok(Self {
            keyboard_input,
            consumer_input,
        })
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
}

pub struct BleHidServer {
    pub hid: HidService,
}

impl BleHidServer {
    pub fn new(softdevice: &mut Softdevice) -> Result<Self, RegisterError> {
        Ok(Self {
            hid: HidService::new(softdevice)?,
        })
    }
}

impl gatt_server::Server for BleHidServer {
    type Event = ();

    fn on_write(
        &self,
        _connection: &Connection,
        _handle: u16,
        _operation: WriteOp,
        _offset: usize,
        _data: &[u8],
    ) -> Option<Self::Event> {
        Some(())
    }
}
