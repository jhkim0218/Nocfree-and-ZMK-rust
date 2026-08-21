#![no_main]
#![no_std]

use embassy_futures::join::{join, join4};
use embassy_futures::select::{Either, select};
use embassy_nrf::bind_interrupts;
use embassy_nrf::gpio::{Level, Output, OutputDrive};
use embassy_nrf::interrupt::{self, InterruptExt, Priority};
use embassy_nrf::peripherals::{TWISPI0, USBD};
use embassy_nrf::pwm::SimplePwm;
use embassy_nrf::saadc::{self, Saadc};
use embassy_nrf::twim::{self, Twim};
use embassy_nrf::usb::vbus_detect::SoftwareVbusDetect;
use embassy_nrf::usb::{self, Driver};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::{Builder, Config};
use nocfree_and_rust::backlight::{BacklightCommand, BacklightState};
use nocfree_and_rust::battery::percent_from_sample;
use nocfree_and_rust::bond_store::{BondStore, SplitSecurity, run_storage};
use nocfree_and_rust::hardware_scanner::{self, KeyState};
use nocfree_and_rust::pca9555::Pca9555Bus;
use nocfree_and_rust::platform::{
    cdc_recovery, enable_usb_power_events, reboot_to_bootloader, softdevice_config,
    update_usb_power,
};
use nocfree_and_rust::scanner::Half;
use nocfree_and_rust::split_ble::{SplitServer, SplitServerEvent, SplitServiceEvent};
use nocfree_and_rust::split_protocol::{
    COMMAND_BACKLIGHT_DOWN, COMMAND_BACKLIGHT_TOGGLE, COMMAND_BACKLIGHT_UP,
    COMMAND_BATTERY_REQUEST, COMMAND_BOOTLOADER, SERVICE_UUID_LE,
};
use nrf_softdevice::Softdevice;
use nrf_softdevice::ble::advertisement_builder::{
    Flag, LegacyAdvertisementBuilder, LegacyAdvertisementPayload, ServiceList,
};
use nrf_softdevice::ble::{gatt_server, peripheral};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    TWISPI0 => twim::InterruptHandler<TWISPI0>;
    SAADC => saadc::InterruptHandler;
});

static KEY_STATE: KeyState<32> = KeyState::new();
static BONDS: BondStore = BondStore::new();
static SPLIT_SECURITY: SplitSecurity = SplitSecurity::new(&BONDS);
static BACKLIGHT_CONTROL: Signal<CriticalSectionRawMutex, BacklightCommand> = Signal::new();
static BATTERY_REQUEST: Signal<CriticalSectionRawMutex, ()> = Signal::new();
static BATTERY_UPDATE: Signal<CriticalSectionRawMutex, u8> = Signal::new();

async fn sample_battery(saadc: &mut Saadc<'_, 1>, enable: &mut Output<'_>) -> u8 {
    enable.set_high();
    Timer::after(Duration::from_millis(5)).await;
    let mut total = 0_i32;
    for _ in 0..8 {
        let mut sample = [0_i16; 1];
        saadc.sample(&mut sample).await;
        total += sample[0] as i32;
    }
    enable.set_low();
    percent_from_sample((total / 8) as i16)
}

async fn run_hardware(
    mut pwm: SimplePwm<'_>,
    _backlight_enable: Output<'_>,
    mut battery_enable: Output<'_>,
    mut saadc: Saadc<'_, 1>,
) -> ! {
    pwm.set_period(10_000);
    let mut state = BacklightState::default();
    pwm.set_duty(0, state.duty(pwm.max_duty()));
    saadc.calibrate().await;
    loop {
        match select(BACKLIGHT_CONTROL.wait(), BATTERY_REQUEST.wait()).await {
            Either::First(command) => {
                state.apply(command);
                pwm.set_duty(0, state.duty(pwm.max_duty()));
            }
            Either::Second(()) => {
                BATTERY_UPDATE.signal(sample_battery(&mut saadc, &mut battery_enable).await)
            }
        }
    }
}

async fn notify_split_state(
    connection: &nrf_softdevice::ble::Connection,
    server: &SplitServer,
) -> ! {
    KEY_STATE.replace(KEY_STATE.latest());
    loop {
        match select(KEY_STATE.wait_changed(), BATTERY_UPDATE.wait()).await {
            Either::First(state) => {
                let state = state.to_le_bytes();
                while server.split.state_notify(connection, &state).is_err() {
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
        BATTERY_REQUEST.signal(());

        let server_run = gatt_server::run(&connection, server, |event| match event {
            SplitServerEvent::Split(SplitServiceEvent::CommandWrite(command))
                if command == COMMAND_BOOTLOADER =>
            {
                reboot_to_bootloader()
            }
            SplitServerEvent::Split(SplitServiceEvent::CommandWrite(command)) => {
                let control = match command {
                    COMMAND_BACKLIGHT_TOGGLE => Some(BacklightCommand::Toggle),
                    COMMAND_BACKLIGHT_DOWN => Some(BacklightCommand::Down),
                    COMMAND_BACKLIGHT_UP => Some(BacklightCommand::Up),
                    _ => None,
                };
                if let Some(control) = control {
                    BACKLIGHT_CONTROL.signal(control);
                } else if command == COMMAND_BATTERY_REQUEST {
                    BATTERY_REQUEST.signal(());
                }
            }
            SplitServerEvent::Split(SplitServiceEvent::StateCccdWrite {
                notifications: true,
            }) => KEY_STATE.replace(KEY_STATE.latest()),
            _ => {}
        });
        match select(server_run, notify_split_state(&connection, server)).await {
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
    interrupt::SAADC.set_priority(Priority::P3);

    let softdevice = Softdevice::enable(&softdevice_config(b"NocFree Rust Right"));
    let split_server = SplitServer::new(softdevice).unwrap();
    let flash = nrf_softdevice::Flash::take(softdevice);
    let (usb_detected, power_ready) = enable_usb_power_events();
    static VBUS: StaticCell<SoftwareVbusDetect> = StaticCell::new();
    let vbus = VBUS.init(SoftwareVbusDetect::new(usb_detected, power_ready));
    let backlight_enable = Output::new(peripherals.P0_05, Level::Low, OutputDrive::Standard);
    let backlight_pwm = SimplePwm::new_1ch(peripherals.PWM2, peripherals.P0_20);
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
                cdc_recovery(cdc),
                hardware_scanner::run(Half::Right, expanders, &KEY_STATE),
                run_split_peripheral(softdevice, &split_server),
            ),
            join(
                run_storage(flash, &BONDS),
                run_hardware(
                    backlight_pwm,
                    backlight_enable,
                    battery_enable,
                    battery_saadc,
                ),
            ),
        ),
    )
    .await;
}
