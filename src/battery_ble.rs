use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::gatt_server::builder::ServiceBuilder;
use nrf_softdevice::ble::gatt_server::characteristic::{Attribute, Metadata, Properties};
use nrf_softdevice::ble::gatt_server::{self, RegisterError};
use nrf_softdevice::ble::{Connection, DeferredReadReply, GattError, SecurityMode, Uuid};

use crate::battery_telemetry::{BatterySnapshot, SNAPSHOT_BYTES};

const SERVICE_UUID_LE: [u8; 16] = 0xf3641600_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();
const STATUS_UUID_LE: [u8; 16] = 0xf3641601_00b0_4240_ba50_05ca45bf8abc_u128.to_le_bytes();

pub struct BatteryService {
    status_value: u16,
    status_cccd: u16,
    level_value: u16,
    level_cccd: u16,
    snapshot: fn() -> BatterySnapshot,
    level: fn() -> u8,
}

impl BatteryService {
    pub fn new(
        softdevice: &mut Softdevice,
        snapshot: fn() -> BatterySnapshot,
        level: fn() -> u8,
    ) -> Result<Self, RegisterError> {
        let mut service = ServiceBuilder::new(softdevice, Uuid::new_128(&SERVICE_UUID_LE))?;
        let status = service
            .add_characteristic(
                Uuid::new_128(&STATUS_UUID_LE),
                Attribute::new(snapshot().encode())
                    .security(SecurityMode::JustWorks)
                    .deferred_read(),
                Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
            )?
            .build();
        service.build();
        let mut service = ServiceBuilder::new(softdevice, Uuid::new_16(0x180f))?;
        let battery_level = service
            .add_characteristic(
                Uuid::new_16(0x2a19),
                Attribute::new([level().min(100)])
                    .security(SecurityMode::JustWorks)
                    .deferred_read(),
                Metadata::new(Properties::new().read().notify()).security(SecurityMode::JustWorks),
            )?
            .build();
        service.build();
        Ok(Self {
            status_value: status.value_handle,
            status_cccd: status.cccd_handle,
            level_value: battery_level.value_handle,
            level_cccd: battery_level.cccd_handle,
            snapshot,
            level,
        })
    }

    pub fn on_deferred_read(&self, handle: u16, offset: usize, reply: DeferredReadReply) {
        let snapshot = (self.snapshot)().encode();
        let level = [(self.level)().min(100)];
        let value: &[u8] = if handle == self.status_value {
            &snapshot
        } else if handle == self.level_value {
            &level
        } else {
            let _ = reply.reply(Err(GattError::ATTERR_ATTRIBUTE_NOT_FOUND));
            return;
        };
        // Reply updates the full attribute at offset zero; SoftDevice applies the read offset.
        let response = if offset <= value.len() {
            Ok(Some(value))
        } else {
            Err(GattError::ATTERR_INVALID_OFFSET)
        };
        let _ = reply.reply(response);
    }

    pub fn notify_snapshot(
        &self,
        connection: &Connection,
        snapshot: &BatterySnapshot,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(connection, self.status_value, &snapshot.encode())
    }

    pub fn notify_level(
        &self,
        connection: &Connection,
        level: u8,
    ) -> Result<(), gatt_server::NotifyValueError> {
        gatt_server::notify_value(connection, self.level_value, &[level.min(100)])
    }

    pub fn subscription_changed(&self, handle: u16, data: &[u8]) -> bool {
        (handle == self.status_cccd || handle == self.level_cccd) && matches!(data, [0, 0] | [1, 0])
    }
}

#[nrf_softdevice::gatt_client(uuid = "f3641600-00b0-4240-ba50-05ca45bf8abc")]
pub struct BatteryClient {
    #[characteristic(uuid = "f3641601-00b0-4240-ba50-05ca45bf8abc", read, notify)]
    pub status: [u8; SNAPSHOT_BYTES],
}
