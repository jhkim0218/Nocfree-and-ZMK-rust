use core::cell::Cell;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

use embassy_sync::blocking_mutex::Mutex;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Channel, TrySendError};

pub use crate::output_policy::OutputMode;
use crate::output_policy::{releases, routes};
use crate::report::{KEY_BITMAP_BYTES, KeyboardReport};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReportFrame {
    pub keyboard: KeyboardReport,
    pub consumer: u16,
}

pub struct OutputRouter {
    latest: Mutex<CriticalSectionRawMutex, Cell<ReportFrame>>,
    usb_queue: Channel<CriticalSectionRawMutex, ReportFrame, 16>,
    ble_queue: Channel<CriticalSectionRawMutex, ReportFrame, 16>,
    mode: AtomicU8,
    usb_connected: AtomicBool,
    release_usb: AtomicBool,
    release_ble: AtomicBool,
}

impl Default for OutputRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl OutputRouter {
    pub const fn new() -> Self {
        Self {
            latest: Mutex::new(Cell::new(ReportFrame {
                keyboard: KeyboardReport {
                    modifiers: 0,
                    reserved: 0,
                    keys: [0; KEY_BITMAP_BYTES],
                },
                consumer: 0,
            })),
            usb_queue: Channel::new(),
            ble_queue: Channel::new(),
            mode: AtomicU8::new(OutputMode::Auto as u8),
            usb_connected: AtomicBool::new(false),
            release_usb: AtomicBool::new(false),
            release_ble: AtomicBool::new(false),
        }
    }

    pub fn publish(&self, frame: ReportFrame) {
        self.latest.lock(|latest| latest.set(frame));
        Self::enqueue_latest(&self.usb_queue, frame);
        Self::enqueue_latest(&self.ble_queue, frame);
    }

    pub fn republish(&self) {
        self.publish(self.latest.lock(Cell::get));
    }

    pub fn send_transient(&self, frame: ReportFrame) {
        Self::enqueue_latest(&self.usb_queue, frame);
        Self::enqueue_latest(&self.ble_queue, frame);
    }

    pub fn set_mode(&self, mode: OutputMode) {
        let previous = self.routes();
        self.mode.store(mode as u8, Ordering::Release);
        self.mark_releases(previous, self.routes());
        self.synchronize();
    }

    pub fn set_usb_connected(&self, connected: bool) {
        let previous = self.routes();
        if self.usb_connected.swap(connected, Ordering::AcqRel) != connected {
            self.mark_releases(previous, self.routes());
            self.synchronize();
        }
    }

    pub fn should_send_usb(&self) -> bool {
        self.routes().0
    }

    pub fn should_send_ble(&self) -> bool {
        self.routes().1
    }

    pub fn take_usb_release(&self) -> bool {
        self.release_usb.swap(false, Ordering::AcqRel)
    }

    pub fn take_ble_release(&self) -> bool {
        self.release_ble.swap(false, Ordering::AcqRel)
    }

    pub async fn wait_usb(&self) -> ReportFrame {
        self.usb_queue.receive().await
    }

    pub async fn wait_ble(&self) -> ReportFrame {
        self.ble_queue.receive().await
    }

    pub fn synchronize_usb(&self) {
        self.usb_queue.clear();
        Self::enqueue_latest(&self.usb_queue, self.latest.lock(Cell::get));
    }

    pub fn synchronize_ble(&self) {
        self.ble_queue.clear();
        Self::enqueue_latest(&self.ble_queue, self.latest.lock(Cell::get));
    }

    fn routes(&self) -> (bool, bool) {
        let mode = match self.mode.load(Ordering::Acquire) {
            value if value == OutputMode::Usb as u8 => OutputMode::Usb,
            value if value == OutputMode::Ble as u8 => OutputMode::Ble,
            value if value == OutputMode::Disabled as u8 => OutputMode::Disabled,
            _ => OutputMode::Auto,
        };
        routes(mode, self.usb_connected.load(Ordering::Acquire))
    }

    fn mark_releases(&self, previous: (bool, bool), current: (bool, bool)) {
        let (release_usb, release_ble) = releases(previous, current);
        if release_usb {
            self.release_usb.store(true, Ordering::Release);
        }
        if release_ble {
            self.release_ble.store(true, Ordering::Release);
        }
    }

    fn synchronize(&self) {
        self.usb_queue.clear();
        self.ble_queue.clear();
        self.republish();
    }

    fn enqueue_latest(
        queue: &Channel<CriticalSectionRawMutex, ReportFrame, 16>,
        frame: ReportFrame,
    ) {
        let mut pending = frame;
        loop {
            match queue.try_send(pending) {
                Ok(()) => break,
                Err(TrySendError::Full(frame)) => {
                    pending = frame;
                    let _ = queue.try_receive();
                }
            }
        }
    }
}
