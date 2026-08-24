use crate::split_protocol::{CLOCK_BYTES, STATE_BYTES};

#[nrf_softdevice::gatt_service(uuid = "f3641400-00b0-4240-ba50-05ca45bf8abc")]
pub struct SplitService {
    #[characteristic(
        uuid = "f3641401-00b0-4240-ba50-05ca45bf8abc",
        read,
        notify,
        security = "justworks"
    )]
    pub state: [u8; STATE_BYTES],

    #[characteristic(
        uuid = "f3641402-00b0-4240-ba50-05ca45bf8abc",
        write_without_response,
        security = "justworks"
    )]
    pub command: u8,

    #[characteristic(
        uuid = "f3641403-00b0-4240-ba50-05ca45bf8abc",
        notify,
        security = "justworks"
    )]
    pub battery: u8,

    #[characteristic(
        uuid = "f3641404-00b0-4240-ba50-05ca45bf8abc",
        write,
        security = "justworks"
    )]
    pub backlight: [u8; 4],

    #[characteristic(
        uuid = "f3641405-00b0-4240-ba50-05ca45bf8abc",
        read,
        write,
        security = "justworks"
    )]
    pub clock: [u8; CLOCK_BYTES],
}

#[nrf_softdevice::gatt_server]
pub struct SplitServer {
    pub split: SplitService,
}

#[nrf_softdevice::gatt_client(uuid = "f3641400-00b0-4240-ba50-05ca45bf8abc")]
pub struct SplitClient {
    #[characteristic(uuid = "f3641401-00b0-4240-ba50-05ca45bf8abc", read, notify)]
    pub state: [u8; STATE_BYTES],

    #[characteristic(
        uuid = "f3641402-00b0-4240-ba50-05ca45bf8abc",
        write,
        write_without_response
    )]
    pub command: u8,

    #[characteristic(uuid = "f3641403-00b0-4240-ba50-05ca45bf8abc", notify)]
    pub battery: u8,

    #[characteristic(uuid = "f3641404-00b0-4240-ba50-05ca45bf8abc", write)]
    pub backlight: [u8; 4],

    #[characteristic(uuid = "f3641405-00b0-4240-ba50-05ca45bf8abc", read, write)]
    pub clock: [u8; CLOCK_BYTES],
}
