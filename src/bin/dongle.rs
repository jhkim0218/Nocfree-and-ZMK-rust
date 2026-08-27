#![no_main]
#![no_std]

use core::future::pending;
use core::panic::PanicInfo;

use embassy_futures::join::join3;
use embassy_nrf::bind_interrupts;
use embassy_nrf::pac;
use embassy_nrf::peripherals::USBD;
use embassy_nrf::usb::vbus_detect::{self, HardwareVbusDetect};
use embassy_nrf::usb::{self, Driver};
use embassy_time::{Duration, Timer};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State as CdcState};
use embassy_usb::class::hid::{Config as HidConfig, HidWriter, State as HidState};
use embassy_usb::driver::Driver as UsbDriver;
use embassy_usb::{Builder, Config};
use nocfree_and_rust::usb_descriptor::{
    CONSUMER_REPORT_BYTES, CONSUMER_REPORT_DESCRIPTOR, KEYBOARD_REPORT_BYTES,
    KEYBOARD_REPORT_DESCRIPTOR,
};

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<USBD>;
    CLOCK_POWER => vbus_detect::InterruptHandler;
});

fn reboot_to_bootloader() -> ! {
    pac::POWER
        .gpregret()
        .write_value(pac::power::regs::Gpregret(0x57));
    cortex_m::peripheral::SCB::sys_reset()
}

#[panic_handler]
fn panic_to_bootloader(_info: &PanicInfo) -> ! {
    reboot_to_bootloader()
}

async fn run_cdc_recovery<'d, D: UsbDriver<'d>>(mut cdc: CdcAcmClass<'d, D>) -> ! {
    cdc.wait_connection().await;
    loop {
        if cdc.line_coding().data_rate() == 1200 {
            Timer::after(Duration::from_millis(100)).await;
            if cdc.line_coding().data_rate() == 1200 {
                reboot_to_bootloader();
            }
        }
        Timer::after(Duration::from_millis(20)).await;
    }
}

async fn hold_hid<'d, D: UsbDriver<'d>>(
    keyboard: HidWriter<'d, D, KEYBOARD_REPORT_BYTES>,
    consumer: HidWriter<'d, D, CONSUMER_REPORT_BYTES>,
) -> ! {
    let _writers = (keyboard, consumer);
    pending().await
}

#[embassy_executor::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let peripherals = embassy_nrf::init(Default::default());
    let usb_driver = Driver::new(peripherals.USBD, Irqs, HardwareVbusDetect::new(Irqs));

    let mut usb_config = Config::new(0x2886, 0x8029);
    usb_config.manufacturer = Some("NocFree");
    usb_config.product = Some("NocFree Rust Dongle");
    usb_config.serial_number = Some("RUST-DONGLE");

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 64];
    let mut msos_descriptor = [0; 256];
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

    join3(
        usb_device.run(),
        run_cdc_recovery(cdc),
        hold_hid(keyboard, consumer),
    )
    .await;
}
