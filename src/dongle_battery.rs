use core::cell::OnceCell;

use nrf_softdevice::ble::gatt_client::{
    self, Characteristic, Client, Descriptor, DiscoverError, HvxType,
};
use nrf_softdevice::ble::{Connection, Uuid};

use crate::battery_ble::{BatteryClient, BatteryClientEvent};
use crate::dongle_ble::{DongleClient, DongleClientEvent};

/// One notification dispatcher per connection; telemetry discovery is optional.
pub struct DongleClients {
    pub input: DongleClient,
    pub battery: OnceCell<BatteryClient>,
}

pub enum DongleEvent {
    Input(DongleClientEvent),
    Battery(BatteryClientEvent),
}

impl Client for DongleClients {
    type Event = DongleEvent;

    fn on_hvx(
        &self,
        conn: &Connection,
        kind: HvxType,
        handle: u16,
        data: &[u8],
    ) -> Option<Self::Event> {
        self.input
            .on_hvx(conn, kind, handle, data)
            .map(DongleEvent::Input)
            .or_else(|| {
                self.battery
                    .get()?
                    .on_hvx(conn, kind, handle, data)
                    .map(DongleEvent::Battery)
            })
    }

    fn uuid() -> Uuid {
        DongleClient::uuid()
    }
    fn new_undiscovered(conn: Connection) -> Self {
        Self {
            input: DongleClient::new_undiscovered(conn),
            battery: OnceCell::new(),
        }
    }
    fn discovered_characteristic(
        &mut self,
        characteristic: &Characteristic,
        descriptors: &[Descriptor],
    ) {
        self.input
            .discovered_characteristic(characteristic, descriptors);
    }
    fn discovery_complete(&mut self) -> Result<(), gatt_client::DiscoverError> {
        self.input.discovery_complete()
    }
}

impl DongleClients {
    pub async fn discover_battery(&self, conn: &Connection) -> Result<(), DiscoverError> {
        let battery: BatteryClient = gatt_client::discover(conn).await?;
        let _ = self.battery.set(battery);
        Ok(())
    }
}
