#![no_main]
#![no_std]

use core::panic::PanicInfo;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use embassy_futures::join::{join, join4};
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
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::{Builder, Config};
use nocfree_and_rust::backlight::{BACKLIGHT_PWM_HZ, BacklightSnapshot};
use nocfree_and_rust::battery::{VoltageFilter, millivolts_from_sample, percent_from_millivolts};
use nocfree_and_rust::bond_store::{BondStore, SplitSecurity, run_storage};
use nocfree_and_rust::hardware_scanner::{self, KeyState};
use nocfree_and_rust::pca9555::Pca9555Bus;
use nocfree_and_rust::platform::{
    cdc_recovery, enable_usb_power_events, panic_reboot_to_bootloader, reboot_to_bootloader,
    softdevice_config, update_usb_power,
};
use nocfree_and_rust::scanner::Half;
use nocfree_and_rust::split_ble::{SplitServer, SplitServerEvent, SplitServiceEvent};
use nocfree_and_rust::split_diagnostics::{
    DIAGNOSTIC_CAPACITY, SplitDiagnosticEvent, SplitDiagnosticRole, SplitDiagnostics,
    duration_millis, pack_address, pack_connection_parameters,
};
use nocfree_and_rust::split_protocol::{
    AdvertisingStage, COMMAND_BATTERY_REQUEST, COMMAND_BOOTLOADER, RIGHT_SPLIT_TX_POWER_DBM,
    SERVICE_UUID_LE, STATE_FLAG_RECONCILE, SplitStateFrame, clock_response,
};
use nocfree_and_rust::status_led::{UNKNOWN_BATTERY_PERCENT, low_battery_led_on};
use nrf_softdevice::ble::advertisement_builder::{
    Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList,
};
use nrf_softdevice::ble::{SecurityMode, TxPower, gatt_server, peripheral};
use nrf_softdevice::{Softdevice, raw};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
    SAADC => saadc::InterruptHandler;
});

#[panic_handler]
fn panic_to_bootloader(_info: &PanicInfo) -> ! {
    panic_reboot_to_bootloader()
}

static KEY_STATE: KeyState<32> = KeyState::new();
static BONDS: BondStore = BondStore::new();
static SPLIT_SECURITY: SplitSecurity = SplitSecurity::new(&BONDS);
static BACKLIGHT_CONTROL: Signal<CriticalSectionRawMutex, BacklightSnapshot> = Signal::new();
static BATTERY_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static BATTERY_UPDATE: Signal<CriticalSectionRawMutex, u8> = Signal::new();
static DISCONNECTED_KEY: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static BATTERY_LEVEL: AtomicU8 = AtomicU8::new(UNKNOWN_BATTERY_PERCENT);
static SPLIT_CONNECTED: AtomicBool = AtomicBool::new(false);
static SECURITY_RECORDED: AtomicBool = AtomicBool::new(false);
static SPLIT_DIAGNOSTICS: SplitDiagnostics<DIAGNOSTIC_CAPACITY> = SplitDiagnostics::new();

fn elapsed_millis(start: Instant) -> u16 {
    duration_millis(start.as_millis(), Instant::now().as_millis())
}

fn record_connection_parameters(connection: &nrf_softdevice::ble::Connection) {
    let params = connection.conn_params();
    SPLIT_DIAGNOSTICS.record(
        SplitDiagnosticEvent::ConnectionParameters,
        0,
        connection.att_mtu(),
        pack_connection_parameters(
            params.min_conn_interval,
            params.max_conn_interval,
            params.slave_latency,
            params.conn_sup_timeout,
        ),
    );
}

fn set_connection_tx_power(connection: &nrf_softdevice::ble::Connection) {
    let handle = connection.handle().expect("connected handle");
    assert_eq!(
        unsafe {
            raw::sd_ble_gap_tx_power_set(
                raw::BLE_GAP_TX_POWER_ROLES_BLE_GAP_TX_POWER_ROLE_CONN as u8,
                handle,
                RIGHT_SPLIT_TX_POWER_DBM,
            )
        },
        raw::NRF_SUCCESS
    );
}

fn record_security_ok(start: Instant) {
    if !SECURITY_RECORDED.swap(true, Ordering::AcqRel) {
        SPLIT_DIAGNOSTICS.record(
            SplitDiagnosticEvent::SecurityOk,
            0,
            elapsed_millis(start),
            0,
        );
    }
}

fn secure(connection: &nrf_softdevice::ble::Connection) -> bool {
    matches!(
        connection.security_mode(),
        SecurityMode::JustWorks
            | SecurityMode::Mitm
            | SecurityMode::LescMitm
            | SecurityMode::Signed
            | SecurityMode::SignedMitm
    )
}

async fn observe_security(connection: &nrf_softdevice::ble::Connection, started: Instant) -> ! {
    for _ in 0..250 {
        if secure(connection) {
            record_security_ok(started);
            break;
        }
        Timer::after(Duration::from_millis(20)).await;
    }
    if !SECURITY_RECORDED.load(Ordering::Acquire) {
        SPLIT_DIAGNOSTICS.record(
            SplitDiagnosticEvent::SecurityError,
            1,
            elapsed_millis(started),
            0,
        );
        SECURITY_RECORDED.store(true, Ordering::Release);
    }
    loop {
        Timer::after(Duration::from_secs(3_600)).await;
    }
}

async fn log_disconnected_keys() -> ! {
    loop {
        let state = KEY_STATE.wait_published().await;
        if state != 0 && !SPLIT_CONNECTED.load(Ordering::Acquire) {
            SPLIT_DIAGNOSTICS.record(
                SplitDiagnosticEvent::DisconnectedKey,
                0,
                state.count_ones() as u16,
                state,
            );
            DISCONNECTED_KEY.signal(());
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
    mut battery_enable: Output<'_>,
    mut saadc: Saadc<'_, 1>,
) -> ! {
    pwm.set_period(BACKLIGHT_PWM_HZ);
    let mut backlight = BacklightSnapshot::new();
    pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
    saadc.calibrate().await;
    let mut filter = VoltageFilter::new();
    pwm.set_duty(0, pwm.max_duty());
    let initial = sample_battery(&mut saadc, &mut battery_enable, &mut filter).await;
    BATTERY_LEVEL.store(initial, Ordering::Release);
    BATTERY_UPDATE.signal(initial);
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
            Either3::Second(()) | Either3::Third(()) => {
                pwm.set_duty(0, pwm.max_duty());
                let level = sample_battery(&mut saadc, &mut battery_enable, &mut filter).await;
                BATTERY_LEVEL.store(level, Ordering::Release);
                BATTERY_UPDATE.signal(level);
                pwm.set_duty(0, backlight.state.duty(pwm.max_duty()));
            }
        }
    }
}

async fn run_status_led(mut red: Flex<'_>) -> ! {
    red.set_high();
    red.set_as_input_output(Pull::None, OutputDrive::Standard0Disconnect1);
    let mut tick = 0_u32;
    loop {
        if low_battery_led_on(BATTERY_LEVEL.load(Ordering::Acquire), tick) {
            red.set_low();
        } else {
            red.set_high();
        }
        tick = tick.wrapping_add(1);
        Timer::after(Duration::from_millis(250)).await;
    }
}

async fn notify_split_state(
    connection: &nrf_softdevice::ble::Connection,
    server: &SplitServer,
) -> ! {
    KEY_STATE.replace(KEY_STATE.latest());
    loop {
        match select(KEY_STATE.wait_changed(), BATTERY_UPDATE.wait()).await {
            Either::First(update) => {
                let frame = SplitStateFrame {
                    pressed: update.value,
                    source_micros: update.source_micros,
                    sequence: update.sequence,
                    flags: if update.reconcile {
                        STATE_FLAG_RECONCILE
                    } else {
                        0
                    },
                }
                .encode();
                while server.split.state_notify(connection, &frame).is_err() {
                    Timer::after(Duration::from_millis(20)).await;
                }
            }
            Either::Second(level) => {
                while server.split.battery_notify(connection, &level).is_err() {
                    Timer::after(Duration::from_millis(20)).await;
                }
            }
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

    let mut stage = AdvertisingStage::Fast;
    let mut attempt = 0_u16;
    loop {
        attempt = attempt.wrapping_add(1).max(1);
        SPLIT_CONNECTED.store(false, Ordering::Release);
        let advertising_config = peripheral::Config {
            interval: stage.interval(),
            timeout: stage.timeout(),
            tx_power: TxPower::Plus8dBm,
            ..Default::default()
        };
        let advertising_started = Instant::now();
        SPLIT_DIAGNOSTICS.record(
            SplitDiagnosticEvent::Advertising,
            stage as i8,
            advertising_config.interval.min(u16::MAX as u32) as u16,
            attempt as u64,
        );
        let advertising = peripheral::advertise_pairable(
            softdevice,
            peripheral::ConnectableAdvertisement::ScannableUndirected {
                adv_data: &ADVERTISEMENT,
                scan_data: &SCAN_RESPONSE,
            },
            &advertising_config,
            &SPLIT_SECURITY,
        );
        let connection = match select(advertising, DISCONNECTED_KEY.wait()).await {
            Either::First(Ok(connection)) => connection,
            Either::First(Err(peripheral::AdvertiseError::Timeout)) => {
                stage = stage.next();
                continue;
            }
            Either::First(Err(error)) => {
                let code = match error {
                    peripheral::AdvertiseError::NoFreeConn => 2,
                    peripheral::AdvertiseError::Raw(_) => 3,
                    peripheral::AdvertiseError::Timeout => unreachable!(),
                };
                SPLIT_DIAGNOSTICS.record(
                    SplitDiagnosticEvent::AdvertisingError,
                    code,
                    elapsed_millis(advertising_started),
                    0,
                );
                continue;
            }
            Either::Second(()) => {
                stage = AdvertisingStage::Fast;
                continue;
            }
        };
        stage = AdvertisingStage::Fast;
        DISCONNECTED_KEY.reset();
        set_connection_tx_power(&connection);
        SPLIT_CONNECTED.store(true, Ordering::Release);
        SECURITY_RECORDED.store(false, Ordering::Release);
        SPLIT_DIAGNOSTICS.record(
            SplitDiagnosticEvent::Connected,
            0,
            elapsed_millis(advertising_started),
            pack_address(
                connection.peer_address().flags,
                connection.peer_address().bytes(),
            ),
        );
        record_connection_parameters(&connection);
        let security_started = Instant::now();
        SPLIT_DIAGNOSTICS.record(SplitDiagnosticEvent::SecurityStart, 0, attempt, 0);
        BATTERY_REQUEST.signal(());

        let server_run = gatt_server::run(&connection, server, |event| match event {
            SplitServerEvent::Split(SplitServiceEvent::CommandWrite(command))
                if command == COMMAND_BOOTLOADER =>
            {
                reboot_to_bootloader()
            }
            SplitServerEvent::Split(SplitServiceEvent::CommandWrite(command)) => {
                if command == COMMAND_BATTERY_REQUEST {
                    BATTERY_REQUEST.signal(());
                }
            }
            SplitServerEvent::Split(SplitServiceEvent::BacklightWrite(bytes)) => {
                if let Some(snapshot) = BacklightSnapshot::decode(bytes) {
                    BACKLIGHT_CONTROL.signal(snapshot);
                }
            }
            SplitServerEvent::Split(SplitServiceEvent::ClockWrite(request)) => {
                let response = clock_response(request, Instant::now().as_micros());
                let _ = server.split.clock_set(&response);
            }
            SplitServerEvent::Split(SplitServiceEvent::StateCccdWrite {
                notifications: true,
            }) => {
                record_security_ok(security_started);
                KEY_STATE.replace(KEY_STATE.latest());
            }
            _ => {}
        });
        match select(
            server_run,
            join(
                notify_split_state(&connection, server),
                observe_security(&connection, security_started),
            ),
        )
        .await
        {
            Either::First(_) => {}
            Either::Second(never) => match never {},
        }
        SPLIT_CONNECTED.store(false, Ordering::Release);
        if !SECURITY_RECORDED.load(Ordering::Acquire) {
            SPLIT_DIAGNOSTICS.record(
                SplitDiagnosticEvent::SecurityError,
                2,
                elapsed_millis(security_started),
                0,
            );
        }
        let reason = connection.disconnect_reason().map(u8::from).unwrap_or(0);
        SPLIT_DIAGNOSTICS.record(SplitDiagnosticEvent::Disconnected, reason as i8, 0, 0);
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
    interrupt::SAADC.set_priority(Priority::P3);

    let softdevice = Softdevice::enable(&softdevice_config(b"NocFree Rust Right"));
    let split_server = SplitServer::new(softdevice).unwrap();
    let flash = nrf_softdevice::Flash::take(softdevice);
    let (usb_detected, power_ready) = enable_usb_power_events();
    static VBUS: StaticCell<SoftwareVbusDetect> = StaticCell::new();
    let vbus = VBUS.init(SoftwareVbusDetect::new(usb_detected, power_ready));
    let backlight_pwm = SimplePwm::new_1ch(peripherals.PWM2, peripherals.P0_20);
    let red_status = Flex::new(peripherals.P0_17);
    let key_interrupt = Input::new(peripherals.P0_05, Pull::Up);
    let battery_enable = Output::new(peripherals.P0_31, Level::Low, OutputDrive::Standard);
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
                cdc_recovery(cdc, &SPLIT_DIAGNOSTICS, SplitDiagnosticRole::Right),
                join(
                    hardware_scanner::run(Half::Right, expanders, key_interrupt, &KEY_STATE),
                    log_disconnected_keys(),
                ),
                run_split_peripheral(softdevice, &split_server),
            ),
            join(
                run_storage(flash, &BONDS),
                join(
                    run_hardware(backlight_pwm, battery_enable, battery_saadc),
                    run_status_led(red_status),
                ),
            ),
        ),
    )
    .await;
}
