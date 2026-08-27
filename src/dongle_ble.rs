use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{self, RegisterError};
use nrf_softdevice::ble::{Connection, SecurityMode, Uuid};

use crate::dongle_protocol::{REPORT_BYTES, SERVICE_UUID_LE};

const REPORT_UUID_LE: [u8; 16] = 0xf3641501_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();

pub struct DongleService {
    report_value: u16,
    report_cccd: u16,
}

impl DongleService {
    pub fn new(softdevice: &mut Softdevice) -> Result<Self, RegisterError> {
        let mut service = ServiceBuilder::new(softdevice, Uuid::new_128(&SERVICE_UUID_LE))?;
        let handles = service
            .add_characteristic(
                Uuid::new_128(&REPORT_UUID_LE),
                Attribute::new([0_u8; REPORT_BYTES]).security(SecurityMode::JustWorks),
                Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
            )?
            .build();
        service.build();
        Ok(Self {
            report_value: handles.value_handle,
            report_cccd: handles.cccd_handle,
        })
    }

    pub fn notify_report(
        &self,
        connection: &Connection,
        report: &[u8; REPORT_BYTES],
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(connection, self.report_value, report)
    }

    pub fn notifications_enabled(&self, handle: u16, data: &[u8]) -> bool {
        handle == self.report_cccd && data == [1, 0]
    }
}

#[nrf_softdevice::gatt_client(uuid = "f3641500-00b0-4240-ba50-05ca45bf8abc")]
pub struct DongleClient {
    #[characteristic(uuid = "f3641501-00b0-4240-ba50-05ca45bf8abc", read, notify)]
    pub report: [u8; REPORT_BYTES],
}
