#![no_main]
#![no_std]

use embassy_futures::join::{join, join4};
use embassy_futures::select::{Either, select};
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::peripherals::{TWISPI0, USBD};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::{Builder, Config};
use nocfree_and_rust::bond_store::{BondStore, SplitSecurity, run_storage};
use nocfree_and_rust::hardware_scanner::{self, KeyState};
use nocfree_and_rust::pca9555::Pca9555Bus;
use nocfree_and_rust::platform::{
    cdc_recovery, enable_usb_power_events, reboot_to_bootloader, softdevice_config,
    update_usb_power,
};
use nocfree_and_rust::scanner::Half;
use nocfree_and_rust::split_ble::{SplitServer, SplitServerEvent, SplitServiceEvent};
use nocfree_and_rust::split_protocol::{COMMAND_BOOTLOADER, SERVICE_UUID_LE};
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::advertisement_builder::{
    Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList,
};
use nrf_softdevice::ble::{gatt_server, peripheral};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
});

static KEY_STATE: KeyState<32> = KeyState::new();
static BONDS: BondStore = BondStore::new();
static SPLIT_SECURITY: SplitSecurity = SplitSecurity::new(&BONDS);

async fn notify_key_state(connection: &nrf_softdevice::ble::Connection, server: &SplitServer) -> ! {
    KEY_STATE.replace(KEY_STATE.latest());
    loop {
        let state = KEY_STATE.wait_changed().await.to_le_bytes();
        while server.split.state_notify(connection, &state).is_err() {
            Timer::after(Duration::from_millis(20)).await;
        }
    }
}

async fn run_split_peripheral(softdevice: &Softdevice, server: &SplitServer) -> ! {
    BONDS.wait_ready().await;
    static ADVERTISEMENT: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_128(ServiceList::Complete, &[SERVICE_UUID_LE])
        .build();
    static SCAN_RESPONSE: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .full_name("NocFree Rust Right")
        .build();

    loop {
        let connection = match peripheral::advertise_pairable(
            softdevice,
            peripheral::ConnectableAdvertisement::ScannableUndirected {
                adv_data: &ADVERTISEMENT,
                scan_data: &SCAN_RESPONSE,
            },
            &peripheral::Config::default(),
            &SPLIT_SECURITY,
        )
        .await
        {
            Ok(connection) => connection,
            Err(_) => continue,
        };

        let server_run = gatt_server::run(&connection, server, |event| match event {
            SplitServerEvent::Split(SplitServiceEvent::CommandWrite(command))
                if command == COMMAND_BOOTLOADER =>
            {
                reboot_to_bootloader()
            }
            SplitServerEvent::Split(SplitServiceEvent::StateCccdWrite {
                notifications: true,
            }) => KEY_STATE.replace(KEY_STATE.latest()),
            _ => {}
        });
        match select(server_run, notify_key_state(&connection, server)).await {
            Either::First(_) => {}
            Either::Second(never) => match never {},
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.gpiote_interrupt_priority = Priority::P2;
    nrf_config.time_interrupt_priority = Priority::P2;
    let peripherals = embassy_nrf::init(nrf_config);
    interrupt::USBD.set_priority(Priority::P2);
    interrupt::TWISPI0.set_priority(Priority::P3);

    let softdevice = Softdevice::enable(&softdevice_config(b"NocFree Rust Right"));
    let split_server = SplitServer::new(softdevice).unwrap();
    let flash = nrf_softdevice::Flash::take(softdevice);
    let (usb_detected, power_ready) = enable_usb_power_events();
    static VBUS: StaticCell<SoftwareVbusDetect> = StaticCell::new();
    let vbus = VBUS.init(SoftwareVbusDetect::new(usb_detected, power_ready));

    let mut twim_buffer = [0_u8; 3];
    let twim = Twim::new(
        peripherals.TWISPI0,
        Irqs,
        peripherals.P0_11,
        peripherals.P1_09,
        twim::Config::default(),
        &mut twim_buffer,
    );
    let expanders = Pca9555Bus::new(twim);

    let usb_driver = Driver::new(peripherals.USBD, Irqs, &*vbus);
    let mut usb_config = Config::new(0x1d50, 0x615e);
    usb_config.manufacturer = Some("NocFree");
    usb_config.product = Some("NocFree Rust Right");
    usb_config.serial_number = Some("RUST-RIGHT");
    let mut config_descriptor = [0; 128];
    let mut bos_descriptor = [0; 32];
    let mut msos_descriptor = [0; 0];
    let mut control_buf = [0; 64];
    let mut cdc_state = CdcState::new();
    let mut usb_builder = Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    let cdc = CdcAcmClass::new(&mut usb_builder, &mut cdc_state, 64);
    let mut usb_device = usb_builder.build();

    join(
        softdevice.run_with_callback(|event| update_usb_power(vbus, event)),
        join(
            join4(
                usb_device.run(),
                cdc_recovery(cdc),
                hardware_scanner::run(Half::Right, expanders, &KEY_STATE),
                run_split_peripheral(softdevice, &split_server),
            ),
            run_storage(flash, &BONDS),
        ),
    )
    .await;
}
