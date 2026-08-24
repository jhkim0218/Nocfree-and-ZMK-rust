use core::cell::Cell;

use embassy_futures::select::select;
use embassy_nrf::Peri;
use embassy_nrf::gpio::{Flex, Input, OutputDrive, Pin, Pull};
use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, TrySendError};
use embassy_sync::signal::Signal;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal_async::i2c::I2c;

use crate::pca9555::Pca9555Bus;
use crate::scanner::{
    ACTIVE_SCAN_MS, Debouncer, Half, IDLE_SAFETY_SCAN_MS, decode_pressed, failure_backoff_ms,
};

pub async fn recover_i2c_bus(sda: Peri<'_, impl Pin>, scl: Peri<'_, impl Pin>) {
    let mut sda = Flex::new(sda);
    let mut scl = Flex::new(scl);
    for pin in [&mut sda, &mut scl] {
        pin.set_high();
        pin.set_as_input_output(Pull::Up, OutputDrive::HighDrive0Disconnect1);
    }
    Timer::after(Duration::from_micros(5)).await;

    if sda.is_low() {
        for _ in 0..9 {
            scl.set_low();
            Timer::after(Duration::from_micros(5)).await;
            scl.set_high();
            Timer::after(Duration::from_micros(5)).await;
            if sda.is_high() {
                break;
            }
        }
    }

    scl.set_low();
    Timer::after(Duration::from_micros(5)).await;
    sda.set_low();
    Timer::after(Duration::from_micros(5)).await;
    scl.set_high();
    Timer::after(Duration::from_micros(5)).await;
    sda.set_high();
    Timer::after(Duration::from_micros(5)).await;
}

pub struct KeyState<const N: usize> {
    latest: Mutex<CriticalSectionRawMutex, Cell<u64>>,
    changed: Channel<CriticalSectionRawMutex, u64, N>,
    published: Signal<CriticalSectionRawMutex, u64>,
}

impl<const N: usize> Default for KeyState<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> KeyState<N> {
    pub const fn new() -> Self {
        Self {
            latest: Mutex::new(Cell::new(0)),
            changed: Channel::new(),
            published: Signal::new(),
        }
    }

    pub fn publish(&self, value: u64) {
        self.latest.lock(|latest| latest.set(value));
        self.published.signal(value);
        let mut pending = value;
        loop {
            match self.changed.try_send(pending) {
                Ok(()) => break,
                Err(TrySendError::Full(value)) => {
                    pending = value;
                    let _ = self.changed.try_receive();
                }
            }
        }
    }

    pub fn latest(&self) -> u64 {
        self.latest.lock(Cell::get)
    }

    pub async fn wait_changed(&self) -> u64 {
        self.changed.receive().await
    }

    pub async fn wait_published(&self) -> u64 {
        self.published.wait().await
    }

    pub fn republish(&self) {
        self.publish(self.latest());
    }

    pub fn replace(&self, value: u64) {
        self.changed.clear();
        self.publish(value);
    }
}

pub async fn run<I, const N: usize>(
    half: Half,
    mut expanders: Pca9555Bus<I>,
    mut interrupt: Input<'_>,
    state: &KeyState<N>,
) -> !
where
    I: I2c,
{
    let mut fail_streak = 0_u8;
    while expanders.configure_and_verify().await.is_err() {
        fail_streak = fail_streak.saturating_add(1);
        Timer::after(Duration::from_millis(failure_backoff_ms(fail_streak) as u64)).await;
    }

    let mut debounce = Debouncer::<47>::default();
    let mut previous_scan = Instant::now();
    fail_streak = 0;

    loop {
        let scan_started = Instant::now();
        let elapsed_ms = scan_started
            .duration_since(previous_scan)
            .as_millis()
            .clamp(1, u16::MAX as u64) as u16;
        previous_scan = scan_started;

        let words = match expanders.read_inputs().await {
            Ok(words) => {
                fail_streak = 0;
                words
            }
            Err(_) => {
                fail_streak = fail_streak.saturating_add(1);
                Timer::after(Duration::from_millis(failure_backoff_ms(fail_streak) as u64)).await;
                continue;
            }
        };

        let raw = decode_pressed(half, words) as u128;
        let update = debounce.update(raw, elapsed_ms);
        if update.changed != 0 {
            state.publish(update.pressed as u64);
        }

        if update.active {
            Timer::at(scan_started + Duration::from_millis(ACTIVE_SCAN_MS as u64)).await;
        } else {
            let _ = select(
                interrupt.wait_for_low(),
                Timer::after(Duration::from_millis(IDLE_SAFETY_SCAN_MS as u64)),
            )
            .await;
        }
    }
}
