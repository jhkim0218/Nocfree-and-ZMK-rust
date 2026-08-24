#![no_main]
#![no_std]

use core::cell::RefCell;
use core::slice;
use core::sync::atomic::{AtomicU8, Ordering};

use embassy_futures::join::{join, join5};
use embassy_futures::select::{Either, Either3, select, select3};
use embassy_nrf::bind_interrupts;
use embassy_nrf::gpio::{Flex, Input, Level, Output, OutputDrive, Pull};
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::peripherals::{TWISPI0, USBD};
use embassy_nrf::pwm::SimplePwm;
use embassy_nrf::saadc::{self, Saadc};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer, with_timeout};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::class::hid::{Config as HidConfig, HidWriter, State as HidState};
use embassy_usb::driver::Driver as UsbDriver;
use embassy_usb::{Builder, Config, Handler};
use nocfree_and_rust::backlight::{AUTO_OFF_SECS, BacklightCommand, BacklightSnapshot};
use nocfree_and_rust::battery::{VoltageFilter, millivolts_from_sample, percent_from_millivolts};
use nocfree_and_rust::battery_status::{BatteryLevels, StatusText, key_report, usage_report};
use nocfree_and_rust::ble_hid::BleHidServer;
use nocfree_and_rust::bond_store::{BondStore, SplitSecurity, run_storage};
use nocfree_and_rust::hardware_scanner::{self, KeyState};
use nocfree_and_rust::link_usb::{LinkUsbClass, State as LinkUsbState};
use nocfree_and_rust::output_policy::physical_switch_mode;
use nocfree_and_rust::output_router::{OutputMode, OutputRouter, ReportFrame};
use nocfree_and_rust::pca9555::Pca9555Bus;
use nocfree_and_rust::platform::{
    cdc_recovery, enable_usb_power_events, key_wake_ready, reboot_application,
    reboot_to_bootloader, softdevice_config, try_system_off, update_usb_power, usb_power_detected,
};
use nocfree_and_rust::power_policy::{DEEP_SLEEP_PREP_MS, DEEP_SLEEP_SECS, should_system_off};
use nocfree_and_rust::report::{Command, ReportEngine};
use nocfree_and_rust::scanner::{Half, SnapshotMerger, decode_half_state, encode_half_state};
use nocfree_and_rust::split_ble::{SplitClient, SplitClientEvent};
use nocfree_and_rust::split_protocol::{
    COMMAND_BATTERY_REQUEST, COMMAND_BOOTLOADER, CONNECTION_INTERVAL_UNITS, CONNECTION_LATENCY,
    CONNECTION_TIMEOUT_UNITS, advertisement_has_split_service,
};
use nocfree_and_rust::status_led::{UNKNOWN_BATTERY_PERCENT, low_battery_led_on, pairing_led_on};
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
use nrf_softdevice::{RawError, Softdevice, raw};
use static_cell::StaticCell;

const KEYBOARD_APPEARANCE: u16 = raw::BLE_APPEARANCE_HID_KEYBOARD as u16;
const KEYBOARD_APPEARANCE_BYTES: [u8; 2] = KEYBOARD_APPEARANCE.to_le_bytes();
const PROFILE_NAMES: [&[u8]; 3] = [b"NocFree 1", b"NocFree 2", b"NocFree 3"];

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
    SAADC => saadc::InterruptHandler;
});

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BleControl {
    Select(u8),
    Pair(u8),
    Clear,
    OutputChanged,
}

static INPUT_STATE: KeyState<32> = KeyState::new();
static OUTPUT: OutputRouter = OutputRouter::new();
static SPLIT_COMMAND: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static BLE_CONTROL: Signal<CriticalSectionRawMutex, BleControl> = Signal::new();
static BACKLIGHT_STATE: Mutex<CriticalSectionRawMutex, RefCell<BacklightSnapshot>> =
    Mutex::new(RefCell::new(BacklightSnapshot::new()));
static BACKLIGHT_CONTROL: Signal<CriticalSectionRawMutex, BacklightSnapshot> = Signal::new();
static SPLIT_BACKLIGHT: Signal<CriticalSectionRawMutex, BacklightSnapshot> = Signal::new();
static BACKLIGHT_ACTIVITY: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static POWER_ACTIVITY: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static BATTERY_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static BATTERY_STATUS: Signal<CriticalSectionRawMutex, BatteryLevels> = Signal::new();
static RIGHT_BATTERY_UPDATE: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static LEFT_BATTERY: AtomicU8 = AtomicU8::new(UNKNOWN_BATTERY_PERCENT);
static RIGHT_BATTERY: AtomicU8 = AtomicU8::new(UNKNOWN_BATTERY_PERCENT);
static KEY_TAP: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static BONDS: BondStore = BondStore::new();
static SPLIT_SECURITY: SplitSecurity = SplitSecurity::new(&BONDS);

struct UsbStatus;

impl Handler for UsbStatus {
    fn reset(&mut self) {
        set_usb_connected(false);
    }

    fn configured(&mut self, configured: bool) {
        set_usb_connected(configured);
    }
}

fn set_usb_connected(connected: bool) {
    let was_enabled = OUTPUT.should_send_ble();
    OUTPUT.set_usb_connected(connected);
    if was_enabled != OUTPUT.should_send_ble() {
        BLE_CONTROL.signal(BleControl::OutputChanged);
    }
}

fn set_output_mode(mode: OutputMode) {
    let was_enabled = OUTPUT.should_send_ble();
    OUTPUT.set_mode(mode);
    if was_enabled != OUTPUT.should_send_ble() {
        BLE_CONTROL.signal(BleControl::OutputChanged);
    }
}

async fn run_mode_switch(ble_detect: Input<'_>, receiver_detect: Input<'_>) -> ! {
    let mut applied = None;
    loop {
        let observed = physical_switch_mode(ble_detect.is_low(), receiver_detect.is_low());
        Timer::after(Duration::from_millis(20)).await;
        let confirmed = physical_switch_mode(ble_detect.is_low(), receiver_detect.is_low());
        if observed == confirmed && confirmed != applied {
            if let Some(mode) = confirmed {
                set_output_mode(mode);
            }
            applied = confirmed;
        }
    }
}

async fn process_key_states() -> ! {
    BONDS.wait_ready().await;
    let mut engine = ReportEngine::default();
    let mut merger = SnapshotMerger::default();
    let mut snapshot = 0;
    loop {
        if let Either::First(encoded) = select(
            INPUT_STATE.wait_changed(),
            Timer::after(Duration::from_millis(20)),
        )
        .await
        {
            let previous = snapshot;
            let (half, state) = decode_half_state(encoded);
            snapshot = merger.update(half, state);
            if snapshot & !previous != 0 {
                BACKLIGHT_ACTIVITY.signal(());
                POWER_ACTIVITY.signal(());
            }
        }
        let effects =
            engine.apply_snapshot_with_at(snapshot, Instant::now().as_millis(), |layer, raw| {
                BONDS.key_action(layer, raw)
            });
        for command in effects.commands() {
            match *command {
                Command::ResetLeft => reboot_application(),
                Command::BootLeft => reboot_to_bootloader(),
                Command::BootRight => SPLIT_COMMAND.signal(COMMAND_BOOTLOADER),
                Command::ProfileSelect(profile) => BLE_CONTROL.signal(BleControl::Select(profile)),
                Command::ProfilePair(profile) => BLE_CONTROL.signal(BleControl::Pair(profile)),
                Command::ProfileClear => BLE_CONTROL.signal(BleControl::Clear),
                Command::OutputUsb => set_output_mode(OutputMode::Usb),
                Command::OutputBle => set_output_mode(OutputMode::Ble),
                Command::BacklightToggle => set_backlight(BacklightCommand::Toggle),
                Command::BacklightDown => set_backlight(BacklightCommand::Down),
                Command::BacklightUp => set_backlight(BacklightCommand::Up),
                Command::BatteryStatus => BATTERY_REQUEST.signal(()),
                Command::SystemSelect(system) => BONDS.set_system(system),
                Command::KeyTap(key) => KEY_TAP.signal(key),
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

fn current_backlight() -> BacklightSnapshot {
    BACKLIGHT_STATE.lock(|snapshot| *snapshot.borrow())
}

fn set_backlight(command: BacklightCommand) {
    let snapshot = BACKLIGHT_STATE.lock(|state| {
        let mut state = state.borrow_mut();
        state.apply(command);
        *state
    });
    BACKLIGHT_CONTROL.signal(snapshot);
    SPLIT_BACKLIGHT.signal(snapshot);
}

async fn run_backlight_timeout() -> ! {
    loop {
        while with_timeout(
            Duration::from_secs(AUTO_OFF_SECS),
            BACKLIGHT_ACTIVITY.wait(),
        )
        .await
        .is_ok()
        {}
        set_backlight(BacklightCommand::Idle);
        BACKLIGHT_ACTIVITY.wait().await;
        set_backlight(BacklightCommand::Wake);
    }
}

async fn run_deep_sleep() -> ! {
    loop {
        while with_timeout(Duration::from_secs(DEEP_SLEEP_SECS), POWER_ACTIVITY.wait())
            .await
            .is_ok()
        {}
        loop {
            if should_system_off(usb_power_detected(), BONDS.is_pairing(), key_wake_ready(31)) {
                set_backlight(BacklightCommand::Idle);
                Timer::after(Duration::from_millis(DEEP_SLEEP_PREP_MS)).await;
                if POWER_ACTIVITY.try_take().is_some() {
                    break;
                }
                if should_system_off(usb_power_detected(), BONDS.is_pairing(), key_wake_ready(31)) {
                    try_system_off(31);
                }
            }
            if with_timeout(Duration::from_secs(1), POWER_ACTIVITY.wait())
                .await
                .is_ok()
            {
                break;
            }
        }
    }
}

async fn sample_battery(
    saadc: &mut Saadc<'_, 1>,
    enable: &mut Output<'_>,
    filter: &mut VoltageFilter,
) -> u8 {
    enable.set_high();
    Timer::after(Duration::from_millis(5)).await;
    let mut total = 0_i32;
    for _ in 0..8 {
        let mut sample = [0_i16; 1];
        saadc.sample(&mut sample).await;
        total += sample[0] as i32;
    }
    enable.set_low();
    let millivolts = millivolts_from_sample((total / 8) as i16);
    percent_from_millivolts(filter.update(millivolts))
}

async fn run_hardware(
    mut pwm: SimplePwm<'_>,
    mut enable: Output<'_>,
    mut saadc: Saadc<'_, 1>,
) -> ! {
    pwm.set_period(10_000);
    let mut backlight = current_backlight();
    pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
    saadc.calibrate().await;
    let mut filter = VoltageFilter::new();
    pwm.set_duty(0, pwm.max_duty());
    let initial = sample_battery(&mut saadc, &mut enable, &mut filter).await;
    LEFT_BATTERY.store(initial, Ordering::Release);
    pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
    loop {
        match select3(
            BACKLIGHT_CONTROL.wait(),
            BATTERY_REQUEST.wait(),
            Timer::after(Duration::from_secs(60)),
        )
        .await
        {
            Either3::First(snapshot) => {
                backlight = snapshot;
                pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
            }
            Either3::Second(()) => {
                RIGHT_BATTERY_UPDATE.reset();
                SPLIT_COMMAND.signal(COMMAND_BATTERY_REQUEST);
                pwm.set_duty(0, pwm.max_duty());
                let left = sample_battery(&mut saadc, &mut enable, &mut filter).await;
                LEFT_BATTERY.store(left, Ordering::Release);
                pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
                let right = with_timeout(Duration::from_millis(500), RIGHT_BATTERY_UPDATE.wait())
                    .await
                    .unwrap_or_else(|_| RIGHT_BATTERY.load(Ordering::Acquire));
                BATTERY_STATUS.signal(BatteryLevels { left, right });
            }
            Either3::Third(()) => {
                pwm.set_duty(0, pwm.max_duty());
                let level = sample_battery(&mut saadc, &mut enable, &mut filter).await;
                LEFT_BATTERY.store(level, Ordering::Release);
                pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
            }
        }
    }
}

async fn run_status_leds(mut red: Flex<'_>, mut blue: Output<'_>) -> ! {
    red.set_high();
    red.set_as_input_output(Pull::None, OutputDrive::Standard0Disconnect1);
    let mut tick = 0_u32;
    loop {
        if low_battery_led_on(LEFT_BATTERY.load(Ordering::Acquire), tick) {
            red.set_low();
        } else {
            red.set_high();
        }
        if pairing_led_on(BONDS.is_pairing() && OUTPUT.should_send_ble(), tick) {
            blue.set_low();
        } else {
            blue.set_high();
        }
        tick = tick.wrapping_add(1);
        Timer::after(Duration::from_millis(250)).await;
    }
}

async fn run_battery_status_output() -> ! {
    loop {
        let text = StatusText::new(BATTERY_STATUS.wait().await);
        for &byte in text.as_bytes() {
            OUTPUT.send_transient(ReportFrame {
                keyboard: key_report(byte),
                consumer: 0,
            });
            Timer::after(Duration::from_millis(8)).await;
            OUTPUT.send_transient(ReportFrame::default());
            Timer::after(Duration::from_millis(8)).await;
        }
        OUTPUT.republish();
    }
}

async fn run_key_tap_output() -> ! {
    loop {
        OUTPUT.send_transient(ReportFrame {
            keyboard: usage_report(KEY_TAP.wait().await, 0),
            consumer: 0,
        });
        Timer::after(Duration::from_millis(8)).await;
        OUTPUT.send_transient(ReportFrame::default());
        OUTPUT.republish();
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
        BleControl::Select(profile) => BONDS.select(profile),
        BleControl::Pair(profile) => BONDS.pair(profile),
        BleControl::Clear => BONDS.clear_selected(),
        BleControl::OutputChanged => {}
    }
}

fn set_gap_device_name(profile: u8) {
    let name = PROFILE_NAMES[profile.min(2) as usize];
    let write_permission = SecurityMode::NoAccess.into_raw();
    assert_eq!(
        unsafe {
            raw::sd_ble_gap_device_name_set(&write_permission, name.as_ptr(), name.len() as u16)
        },
        raw::NRF_SUCCESS
    );
}

async fn disconnect_ble(connection: &nrf_softdevice::ble::Connection) {
    let _ = connection.disconnect();
    while connection.handle().is_some() {
        Timer::after(Duration::from_millis(5)).await;
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

async fn run_ble_host(softdevice: &Softdevice, server: &BleHidServer) -> ! {
    BONDS.wait_ready().await;
    static ADVERTISEMENT_1: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(
            ServiceList::Complete,
            &[ServiceUuid16::HUMAN_INTERFACE_DEVICE],
        )
        .raw(
            AdvertisementDataType::APPEARANCE,
            &KEYBOARD_APPEARANCE_BYTES,
        )
        .full_name("NocFree 1")
        .build();
    static ADVERTISEMENT_2: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(
            ServiceList::Complete,
            &[ServiceUuid16::HUMAN_INTERFACE_DEVICE],
        )
        .raw(
            AdvertisementDataType::APPEARANCE,
            &KEYBOARD_APPEARANCE_BYTES,
        )
        .full_name("NocFree 2")
        .build();
    static ADVERTISEMENT_3: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new()
        .flags(&[Flag::GeneralDiscovery, Flag::LE_Only])
        .services_16(
            ServiceList::Complete,
            &[ServiceUuid16::HUMAN_INTERFACE_DEVICE],
        )
        .raw(
            AdvertisementDataType::APPEARANCE,
            &KEYBOARD_APPEARANCE_BYTES,
        )
        .full_name("NocFree 3")
        .build();
    static SCAN_RESPONSE: LegacyAdvertisementPayload = LegacyAdvertisementBuilder::new().build();
    loop {
        if !OUTPUT.should_send_ble() || !BONDS.selected_connectable() {
            apply_ble_control(BLE_CONTROL.wait().await);
            continue;
        }
        let profile = BONDS.selected();
        set_gap_device_name(profile);
        let advertisement = match profile {
            0 => &ADVERTISEMENT_1,
            1 => &ADVERTISEMENT_2,
            _ => &ADVERTISEMENT_3,
        };
        let advertising_config = peripheral::Config {
            interval: 160, // 100 ms
            ..Default::default()
        };
        let advertising = peripheral::advertise_pairable(
            softdevice,
            peripheral::ConnectableAdvertisement::ScannableUndirected {
                adv_data: advertisement,
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
        if !BONDS.accepts_connection(&connection)
            || connection.request_security().is_err()
            || !wait_for_security(&connection).await
            || !BONDS.restore_sys_attrs(&connection)
        {
            disconnect_ble(&connection).await;
            continue;
        }

        let control = match select3(
            gatt_server::run(&connection, server, |_| {
                BONDS.capture_sys_attrs(&connection);
                OUTPUT.synchronize_ble();
            }),
            notify_ble_reports(&connection, server),
            BLE_CONTROL.wait(),
        )
        .await
        {
            Either3::First(_) | Either3::Second(()) => None,
            Either3::Third(control) => Some(control),
        };
        if let Some(control) = control {
            disconnect_ble(&connection).await;
            apply_ble_control(control);
        }
        drop(connection);
    }
}

async fn send_split_commands(client: &SplitClient) -> ! {
    let _ = client.backlight_write(&current_backlight().encode()).await;
    loop {
        match select(SPLIT_COMMAND.wait(), SPLIT_BACKLIGHT.wait()).await {
            Either::First(command) => {
                let _ = client.command_write_without_response(&command).await;
            }
            Either::Second(snapshot) => {
                let _ = client.backlight_write(&snapshot.encode()).await;
            }
        }
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
        let had_split_peer = BONDS.has_split_peer();
        if !secure_split_connection(&connection).await {
            if had_split_peer {
                BONDS.clear_split_peer();
            }
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
        if client.battery_cccd_write(true).await.is_err() {
            INPUT_STATE.publish(encode_half_state(Half::Right, 0));
            continue;
        }

        let notifications = gatt_client::run(&connection, &client, |event| match event {
            SplitClientEvent::StateNotification(bytes) => {
                INPUT_STATE.publish(encode_half_state(Half::Right, u64::from_le_bytes(bytes)));
            }
            SplitClientEvent::BatteryNotification(level) => {
                RIGHT_BATTERY.store(level, Ordering::Release);
                RIGHT_BATTERY_UPDATE.signal(level);
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

    wait_for_security(connection).await
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let mut nrf_config = embassy_nrf::config::Config::default();
    nrf_config.gpiote_interrupt_priority = Priority::P2;
    nrf_config.time_interrupt_priority = Priority::P2;
    let peripherals = embassy_nrf::init(nrf_config);
    interrupt::USBD.set_priority(Priority::P2);
    interrupt::TWISPI0.set_priority(Priority::P3);
    interrupt::SAADC.set_priority(Priority::P3);

    let softdevice = Softdevice::enable(&softdevice_config(b"NocFree 1"));
    assert_eq!(
        unsafe { raw::sd_ble_gap_appearance_set(KEYBOARD_APPEARANCE) },
        raw::NRF_SUCCESS
    );
    let ble_hid_server = BleHidServer::new(softdevice).unwrap();
    let flash = nrf_softdevice::Flash::take(softdevice);
    let (usb_detected, power_ready) = enable_usb_power_events();
    static VBUS: StaticCell<SoftwareVbusDetect> = StaticCell::new();
    let vbus = VBUS.init(SoftwareVbusDetect::new(usb_detected, power_ready));
    let ble_detect = Input::new(peripherals.P0_15, Pull::Up);
    let receiver_detect = Input::new(peripherals.P0_17, Pull::Up);
    let key_interrupt = Input::new(peripherals.P0_31, Pull::Up);
    let battery_enable = Output::new(peripherals.P0_05, Level::Low, OutputDrive::Standard);
    let backlight_pwm = SimplePwm::new_1ch(peripherals.PWM2, peripherals.P0_20);
    let red_status = Flex::new(peripherals.P0_09);
    let blue_status = Output::new(peripherals.P0_10, Level::High, OutputDrive::Standard);
    let battery_saadc = Saadc::new(
        peripherals.SAADC,
        Irqs,
        saadc::Config::default(),
        [saadc::ChannelConfig::single_ended(peripherals.P0_04)],
    );

    let mut sda = peripherals.P0_11;
    let mut scl = peripherals.P1_09;
    hardware_scanner::recover_i2c_bus(sda.reborrow(), scl.reborrow()).await;
    let mut twim_buffer = [0_u8; 3];
    let twim = Twim::new(
        peripherals.TWISPI0,
        Irqs,
        sda,
        scl,
        twim::Config::default(),
        &mut twim_buffer,
    );
    let expanders = Pca9555Bus::new(twim);

    let usb_driver = Driver::new(peripherals.USBD, Irqs, &*vbus);
    let mut usb_config = Config::new(0x2886, 0x8029);
    usb_config.manufacturer = Some("NocFree");
    usb_config.product = Some("NocFree & ANSI");
    usb_config.serial_number = Some("RUST-LEFT");
    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 128];
    let mut keyboard_state = HidState::new();
    let mut consumer_state = HidState::new();
    let mut cdc_state = CdcState::new();
    let mut link_state = LinkUsbState::new();
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
    let link = LinkUsbClass::new(&mut usb_builder, &mut link_state);
    let mut usb_device = usb_builder.build();

    join(
        softdevice.run_with_callback(|event| update_usb_power(vbus, event)),
        join(
            join5(
                usb_device.run(),
                cdc_recovery(cdc),
                run_usb_reports(keyboard, consumer),
                hardware_scanner::run(Half::Left, expanders, key_interrupt, &INPUT_STATE),
                link.run(&BONDS),
            ),
            join5(
                process_key_states(),
                run_mode_switch(ble_detect, receiver_detect),
                run_split_central(softdevice),
                run_ble_host(softdevice, &ble_hid_server),
                join(
                    run_storage(flash, &BONDS),
                    join(
                        run_hardware(backlight_pwm, battery_enable, battery_saadc),
                        join(
                            run_status_leds(red_status, blue_status),
                            join(
                                run_backlight_timeout(),
                                join(
                                    run_deep_sleep(),
                                    join(run_battery_status_output(), run_key_tap_output()),
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    )
    .await;
}
