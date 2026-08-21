#![no_main]
#![no_std]

use core::slice;

use embassy_futures::join::{join, join4};
use embassy_futures::select::{Either, Either3, select, select3};
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::peripherals::{TWISPI0, USBD};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::class::hid::{Config as HidConfig, HidWriter, State as HidState};
use embassy_usb::driver::Driver as UsbDriver;
use embassy_usb::{Builder, Config, Handler};
use nocfree_and_rust::ble_hid::BleHidServer;
use nocfree_and_rust::bond_store::{BondStore, SplitSecurity, run_storage};
use nocfree_and_rust::hardware_scanner::{self, KeyState};
use nocfree_and_rust::output_router::{OutputMode, OutputRouter, ReportFrame};
use nocfree_and_rust::pca9555::Pca9555Bus;
use nocfree_and_rust::platform::{
    cdc_recovery, enable_usb_power_events, reboot_application, softdevice_config, update_usb_power,
};
use nocfree_and_rust::report::{Command, ReportEngine};
use nocfree_and_rust::scanner::{Half, SnapshotMerger, decode_half_state, encode_half_state};
use nocfree_and_rust::split_ble::{SplitClient, SplitClientEvent};
use nocfree_and_rust::split_protocol::{
    COMMAND_BOOTLOADER, CONNECTION_INTERVAL_UNITS, CONNECTION_LATENCY, CONNECTION_TIMEOUT_UNITS,
    advertisement_has_split_service,
};
use nocfree_and_rust::usb_descriptor::{
    CONSUMER_REPORT_BYTES, CONSUMER_REPORT_DESCRIPTOR, KEYBOARD_REPORT_BYTES,
    KEYBOARD_REPORT_DESCRIPTOR,
};
use nrf_softdevice::ble::advertisement_builder::{
    AdvertisementDataType, Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload,
    ServiceList, ServiceUuid16,
};
use nrf_softdevice::ble::gatt_server::NotifyValueError;
use nrf_softdevice::ble::{
    Address, EncryptError, SecurityMode, central, gatt_client, gatt_server, peripheral,
};
use nrf_softdevice::{RawError, Softdevice};
use panic_halt as _;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BleControl {
    SelectProfile(u8),
    ClearProfile,
}

static INPUT_STATE: KeyState<32> = KeyState::new();
static OUTPUT: OutputRouter = OutputRouter::new();
static SPLIT_COMMAND: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static BLE_CONTROL: Signal<CriticalSectionRawMutex, BleControl> = Signal::new();
static BONDS: BondStore = BondStore::new();
static SPLIT_SECURITY: SplitSecurity = SplitSecurity::new(&BONDS);

struct UsbStatus;

impl Handler for UsbStatus {
    fn reset(&mut self) {
        OUTPUT.set_usb_connected(false);
    }

    fn configured(&mut self, configured: bool) {
        OUTPUT.set_usb_connected(configured);
    }
}

async fn process_key_states() -> ! {
    let mut engine = ReportEngine::default();
    let mut merger = SnapshotMerger::default();
    loop {
        let (half, state) = decode_half_state(INPUT_STATE.wait_changed().await);
        let effects = engine.apply_snapshot(merger.update(half, state));
        for command in effects.commands() {
            match *command {
                Command::ResetLeft => reboot_application(),
                Command::BootRight => SPLIT_COMMAND.signal(COMMAND_BOOTLOADER),
                Command::ProfileSelect(profile) => {
                    BLE_CONTROL.signal(BleControl::SelectProfile(profile))
                }
                Command::ProfileClear => BLE_CONTROL.signal(BleControl::ClearProfile),
                Command::OutputUsb => OUTPUT.set_mode(OutputMode::Usb),
                Command::OutputBle => OUTPUT.set_mode(OutputMode::Ble),
                Command::None => {}
            }
        }
        if effects.keyboard_changed || effects.consumer_changed {
            OUTPUT.publish(ReportFrame {
                keyboard: effects.keyboard,
                consumer: effects.consumer,
            });
        }
    }
}

async fn run_usb_reports<'d, D: UsbDriver<'d>>(
    mut keyboard: HidWriter<'d, D, KEYBOARD_REPORT_BYTES>,
    mut consumer: HidWriter<'d, D, CONSUMER_REPORT_BYTES>,
) -> ! {
    loop {
        join(keyboard.ready(), consumer.ready()).await;
        OUTPUT.synchronize_usb();
        loop {
            let frame = OUTPUT.wait_usb().await;
            if OUTPUT.take_usb_release()
                && (keyboard
                    .write(ReportFrame::default().keyboard.as_bytes())
                    .await
                    .is_err()
                    || consumer.write(&0_u16.to_le_bytes()).await.is_err())
            {
                OUTPUT.set_usb_connected(false);
                break;
            }
            if !OUTPUT.should_send_usb() {
                continue;
            }
            if keyboard.write(frame.keyboard.as_bytes()).await.is_err()
                || consumer.write(&frame.consumer.to_le_bytes()).await.is_err()
            {
                OUTPUT.set_usb_connected(false);
                break;
            }
        }
    }
}

async fn notify_ble_reports(connection: &nrf_softdevice::ble::Connection, server: &BleHidServer) {
    OUTPUT.synchronize_ble();
    loop {
        let frame = OUTPUT.wait_ble().await;
        if OUTPUT.take_ble_release() {
            let empty = ReportFrame::default();
            if !send_ble_notification(|| server.hid.notify_keyboard(connection, &empty.keyboard))
                .await
                || !send_ble_notification(|| server.hid.notify_consumer(connection, empty.consumer))
                    .await
            {
                return;
            }
        }
        if !OUTPUT.should_send_ble() {
            continue;
        }
        if !send_ble_notification(|| server.hid.notify_keyboard(connection, &frame.keyboard)).await
            || !send_ble_notification(|| server.hid.notify_consumer(connection, frame.consumer))
                .await
        {
            return;
        }
    }
}

async fn send_ble_notification(mut send: impl FnMut() -> Result<(), NotifyValueError>) -> bool {
    loop {
        match send() {
            Ok(()) => return true,
            Err(NotifyValueError::Disconnected) => return false,
            Err(NotifyValueError::Raw(RawError::Resources | RawError::Busy)) => {
                Timer::after(Duration::from_millis(5)).await;
            }
            Err(NotifyValueError::Raw(_)) => return true,
        }
    }
}

fn apply_ble_control(control: BleControl) {
    match control {
        BleControl::SelectProfile(profile) => BONDS.select(profile),
        BleControl::ClearProfile => BONDS.clear_selected(),
    }
}

async fn run_ble_host(softdevice: &Softdevice, server: &BleHidServer) -> ! {
    BONDS.wait_ready().await;
    static ADVERTISEMENT: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(
            ServiceList::Complete,
            &[ServiceUuid16::HUMAN_INTERFACE_DEVICE],
        )
        .raw(AdvertisementDataType::APPEARANCE, &[0xc1, 0x03])
        .full_name("NocFree Rust")
        .build();
    static SCAN_RESPONSE: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new().build();
    loop {
        let advertising_config = peripheral::Config::default();
        let advertising = peripheral::advertise_pairable(
            softdevice,
            peripheral::ConnectableAdvertisement::ScannableUndirected {
                adv_data: &ADVERTISEMENT,
                scan_data: &SCAN_RESPONSE,
            },
            &advertising_config,
            &BONDS,
        );
        let connection = match select(advertising, BLE_CONTROL.wait()).await {
            Either::First(Ok(connection)) => connection,
            Either::First(Err(_)) => continue,
            Either::Second(control) => {
                apply_ble_control(control);
                continue;
            }
        };

        let control = match select3(
            gatt_server::run(&connection, server, |_| OUTPUT.synchronize_ble()),
            notify_ble_reports(&connection, server),
            BLE_CONTROL.wait(),
        )
        .await
        {
            Either3::First(_) | Either3::Second(()) => None,
            Either3::Third(control) => Some(control),
        };
        drop(connection);
        if let Some(control) = control {
            apply_ble_control(control);
        }
    }
}

async fn send_split_commands(client: &SplitClient) -> ! {
    loop {
        let command = SPLIT_COMMAND.wait().await;
        let _ = client.command_write_without_response(&command).await;
    }
}

async fn run_split_central(softdevice: &Softdevice) -> ! {
    BONDS.wait_ready().await;
    loop {
        let address = match central::scan(softdevice, &central::ScanConfig::default(), |report| {
            if report.type_.connectable() == 0 || report.data.len == 0 {
                return None;
            }
            let data =
                unsafe { slice::from_raw_parts(report.data.p_data, report.data.len as usize) };
            advertisement_has_split_service(data).then(|| Address::from_raw(report.peer_addr))
        })
        .await
        {
            Ok(address) => address,
            Err(_) => continue,
        };

        let addresses = [&address];
        let mut connect_config = central::ConnectConfig::default();
        connect_config.scan_config.whitelist = Some(&addresses);
        connect_config.conn_params.min_conn_interval = CONNECTION_INTERVAL_UNITS;
        connect_config.conn_params.max_conn_interval = CONNECTION_INTERVAL_UNITS;
        connect_config.conn_params.slave_latency = CONNECTION_LATENCY;
        connect_config.conn_params.conn_sup_timeout = CONNECTION_TIMEOUT_UNITS;
        let connection = match central::connect_with_security(
            softdevice,
            &connect_config,
            &SPLIT_SECURITY,
        )
        .await
        {
            Ok(connection) => connection,
            Err(_) => continue,
        };
        if !secure_split_connection(&connection).await {
            INPUT_STATE.publish(encode_half_state(Half::Right, 0));
            continue;
        }
        let client: SplitClient = match gatt_client::discover(&connection).await {
            Ok(client) => client,
            Err(_) => {
                INPUT_STATE.publish(encode_half_state(Half::Right, 0));
                continue;
            }
        };
        if client.state_cccd_write(true).await.is_err() {
            INPUT_STATE.publish(encode_half_state(Half::Right, 0));
            continue;
        }

        let notifications = gatt_client::run(&connection, &client, |event| match event {
            SplitClientEvent::StateNotification(bytes) => {
                INPUT_STATE.publish(encode_half_state(Half::Right, u64::from_le_bytes(bytes)));
            }
        });
        match select(notifications, send_split_commands(&client)).await {
            Either::First(_) => {}
            Either::Second(never) => match never {},
        }
        INPUT_STATE.publish(encode_half_state(Half::Right, 0));
    }
}

async fn secure_split_connection(connection: &nrf_softdevice::ble::Connection) -> bool {
    let started = match connection.encrypt() {
        Ok(()) => true,
        Err(EncryptError::PeerKeysNotFound) => connection.request_pairing().is_ok(),
        Err(_) => false,
    };
    if !started {
        return false;
    }

    for _ in 0..250 {
        if matches!(
            connection.security_mode(),
            SecurityMode::JustWorks
                | SecurityMode::Mitm
                | SecurityMode::LescMitm
                | SecurityMode::Signed
                | SecurityMode::SignedMitm
        ) {
            return true;
        }
        Timer::after(Duration::from_millis(20)).await;
    }
    false
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.gpiote_interrupt_priority = Priority::P2;
    nrf_config.time_interrupt_priority = Priority::P2;
    let peripherals = embassy_nrf::init(nrf_config);
    interrupt::USBD.set_priority(Priority::P2);
    interrupt::TWISPI0.set_priority(Priority::P3);

    let softdevice = Softdevice::enable(&softdevice_config(b"NocFree Rust"));
    let ble_hid_server = BleHidServer::new(softdevice).unwrap();
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
    usb_config.product = Some("NocFree Rust");
    usb_config.serial_number = Some("RUST-LEFT");
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 32];
    let mut msos_descriptor = [0; 0];
    let mut control_buf = [0; 128];
    let mut keyboard_state = HidState::new();
    let mut consumer_state = HidState::new();
    let mut cdc_state = CdcState::new();
    let mut usb_status = UsbStatus;
    let mut usb_builder = Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
    usb_builder.handler(&mut usb_status);
    let keyboard = HidWriter::<_, KEYBOARD_REPORT_BYTES>::new(
        &mut usb_builder,
        &mut keyboard_state,
        HidConfig {
            report_descriptor: KEYBOARD_REPORT_DESCRIPTOR,
            request_handler: None,
            poll_ms: 1,
            max_packet_size: KEYBOARD_REPORT_BYTES as u16,
        },
    );
    let consumer = HidWriter::<_, CONSUMER_REPORT_BYTES>::new(
        &mut usb_builder,
        &mut consumer_state,
        HidConfig {
            report_descriptor: CONSUMER_REPORT_DESCRIPTOR,
            request_handler: None,
            poll_ms: 1,
            max_packet_size: CONSUMER_REPORT_BYTES as u16,
        },
    );
    let cdc = CdcAcmClass::new(&mut usb_builder, &mut cdc_state, 64);
    let mut usb_device = usb_builder.build();

    join(
        softdevice.run_with_callback(|event| update_usb_power(vbus, event)),
        join(
            join4(
                usb_device.run(),
                cdc_recovery(cdc),
                run_usb_reports(keyboard, consumer),
                hardware_scanner::run(Half::Left, expanders, &INPUT_STATE),
            ),
            join4(
                process_key_states(),
                run_split_central(softdevice),
                run_ble_host(softdevice, &ble_hid_server),
                run_storage(flash, &BONDS),
            ),
        ),
    )
    .await;
}
