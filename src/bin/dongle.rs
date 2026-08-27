#![no_main]
#![no_std]

use core::slice;

use embassy_futures::join::{join, join4};
use embassy_futures::select::{Either, select};
use embassy_nrf::bind_interrupts;
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::peripherals::USBD;
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, TrySendError};
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::class::hid::{Config as HidConfig, HidWriter, State as HidState};
use embassy_usb::driver::Driver as UsbDriver;
use embassy_usb::{Builder, Config};
use nocfree_and_rust::bond_store::{BondStore, DongleSecurity, run_storage};
use nocfree_and_rust::dongle_ble::{DongleClient, DongleClientEvent};
use nocfree_and_rust::dongle_protocol::{
    ATT_MTU, CONNECTION_INTERVAL_UNITS, CONNECTION_LATENCY, CONNECTION_TIMEOUT_UNITS,
    DongleReportReceiver, advertisement_has_dongle_service,
};
use nocfree_and_rust::output_router::ReportFrame;
use nocfree_and_rust::platform::{
    enable_usb_power_events, reboot_application, reboot_to_bootloader, softdevice_config,
    update_usb_power,
};
use nocfree_and_rust::usb_descriptor::{
    CONSUMER_REPORT_BYTES, CONSUMER_REPORT_DESCRIPTOR, KEYBOARD_REPORT_BYTES,
    KEYBOARD_REPORT_DESCRIPTOR,
};
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::{Address, EncryptError, SecurityMode, central, gatt_client};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
});

static BONDS: BondStore = BondStore::new();
static DONGLE_SECURITY: DongleSecurity = DongleSecurity::new(&BONDS);
static REPORTS: Channel<CriticalSectionRawMutex, ReportFrame, 16> = Channel::new();
static CONTROL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

fn enqueue_report(frame: ReportFrame) {
    let mut pending = frame;
    loop {
        match REPORTS.try_send(pending) {
            Ok(()) => return,
            Err(TrySendError::Full(frame)) => {
                pending = frame;
                let _ = REPORTS.try_receive();
            }
        }
    }
}

async fn run_usb_reports<'d, D: UsbDriver<'d>>(
    mut keyboard: HidWriter<'d, D, KEYBOARD_REPORT_BYTES>,
    mut consumer: HidWriter<'d, D, CONSUMER_REPORT_BYTES>,
) -> ! {
    loop {
        join(keyboard.ready(), consumer.ready()).await;
        REPORTS.clear();
        enqueue_report(ReportFrame::default());
        loop {
            let frame = REPORTS.receive().await;
            if keyboard.write(frame.keyboard.as_bytes()).await.is_err()
                || consumer.write(&frame.consumer.to_le_bytes()).await.is_err()
            {
                break;
            }
        }
    }
}

async fn wait_for_security(connection: &nrf_softdevice::ble::Connection) -> bool {
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

async fn secure_connection(connection: &nrf_softdevice::ble::Connection) -> bool {
    match connection.encrypt() {
        Ok(()) => {}
        Err(EncryptError::PeerKeysNotFound) => {
            if connection.request_pairing().is_err() {
                return false;
            }
        }
        Err(_) => return false,
    }
    wait_for_security(connection).await
}

async fn disconnect(connection: &nrf_softdevice::ble::Connection) {
    let _ = connection.disconnect();
    while connection.handle().is_some() {
        Timer::after(Duration::from_millis(5)).await;
    }
}

async fn run_receiver(softdevice: &Softdevice) -> ! {
    BONDS.wait_ready().await;
    loop {
        let address = match central::scan(softdevice, &central::ScanConfig::default(), |report| {
            if report.type_.connectable() == 0 || report.data.len == 0 {
                return None;
            }
            let data =
                unsafe { slice::from_raw_parts(report.data.p_data, report.data.len as usize) };
            let address = Address::from_raw(report.peer_addr);
            (advertisement_has_dongle_service(data) && BONDS.accepts_dongle_address(address))
                .then_some(address)
        })
        .await
        {
            Ok(address) => address,
            Err(_) => continue,
        };

        let addresses = [&address];
        let mut config = central::ConnectConfig {
            att_mtu: Some(ATT_MTU),
            ..Default::default()
        };
        config.scan_config.whitelist = Some(&addresses);
        config.conn_params.min_conn_interval = CONNECTION_INTERVAL_UNITS;
        config.conn_params.max_conn_interval = CONNECTION_INTERVAL_UNITS;
        config.conn_params.slave_latency = CONNECTION_LATENCY;
        config.conn_params.conn_sup_timeout = CONNECTION_TIMEOUT_UNITS;
        let connection =
            match central::connect_with_security(softdevice, &config, &DONGLE_SECURITY).await {
                Ok(connection) => connection,
                Err(_) => continue,
            };
        let had_dongle_peer = BONDS.has_dongle_peer();
        if !secure_connection(&connection).await {
            if had_dongle_peer {
                BONDS.clear_dongle_peer();
            }
            disconnect(&connection).await;
            continue;
        }
        let client: DongleClient = match gatt_client::discover(&connection).await {
            Ok(client) => client,
            Err(_) => {
                disconnect(&connection).await;
                continue;
            }
        };
        if client.report_cccd_write(true).await.is_err() {
            disconnect(&connection).await;
            continue;
        }

        enqueue_report(ReportFrame::default());
        let mut receiver = DongleReportReceiver::new();
        let notifications = gatt_client::run(&connection, &client, |event| {
            let DongleClientEvent::ReportNotification(bytes) = event;
            if let Some(frame) = receiver.accept(bytes) {
                enqueue_report(frame);
            }
        });
        if let Either::Second(()) = select(notifications, CONTROL.wait()).await {
            disconnect(&connection).await;
        }
        enqueue_report(ReportFrame::default());
        drop(connection);
    }
}

async fn cdc_control<'d, D: UsbDriver<'d>>(mut cdc: CdcAcmClass<'d, D>) -> ! {
    cdc.wait_connection().await;
    let (_sender, receiver, _control) = cdc.split_with_control();
    loop {
        match receiver.line_coding().data_rate() {
            1200 => {
                Timer::after(Duration::from_millis(100)).await;
                if receiver.line_coding().data_rate() == 1200 {
                    reboot_to_bootloader();
                }
            }
            2400 => {
                BONDS.clear_dongle_peer();
                CONTROL.signal(());
                Timer::after(Duration::from_millis(500)).await;
                reboot_application();
            }
            _ => Timer::after(Duration::from_millis(20)).await,
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

    let softdevice = Softdevice::enable(&softdevice_config(b"NocFree Rust Dongle"));
    let flash = nrf_softdevice::Flash::take(softdevice);
    let (usb_detected, power_ready) = enable_usb_power_events();
    static VBUS: StaticCell<SoftwareVbusDetect> = StaticCell::new();
    let vbus = VBUS.init(SoftwareVbusDetect::new(usb_detected, power_ready));

    let usb_driver = Driver::new(peripherals.USBD, Irqs, &*vbus);
    let mut usb_config = Config::new(0x239a, 0x80d8);
    usb_config.manufacturer = Some("NocFree");
    usb_config.product = Some("NocFree Rust Dongle");
    usb_config.serial_number = Some("RUST-DONGLE");
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut msos_descriptor = [0; 0];
    let mut control_buf = [0; 128];
    let mut keyboard_state = HidState::new();
    let mut consumer_state = HidState::new();
    let mut cdc_state = CdcState::new();
    let mut usb_builder = Builder::new(
        usb_driver,
        usb_config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );
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
                cdc_control(cdc),
                run_usb_reports(keyboard, consumer),
                run_receiver(softdevice),
            ),
            run_storage(flash, &BONDS),
        ),
    )
    .await;
}
