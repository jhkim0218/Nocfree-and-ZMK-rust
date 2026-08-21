use core::mem::MaybeUninit;

use embassy_usb::control::{OutResponse, Recipient, Request, RequestType};
use embassy_usb::driver::{Driver, Endpoint, EndpointError, EndpointIn, EndpointOut};
use embassy_usb::msos::{
    CompatibleIdFeatureDescriptor, PropertyData, RegistryPropertyFeatureDescriptor,
};
use embassy_usb::types::InterfaceNumber;
use embassy_usb::{Builder, Handler};

use crate::bond_store::BondStore;
use crate::link_protocol::MAX_FRAME_BYTES;

const PACKET_SIZE: usize = 64;

pub struct State {
    control: MaybeUninit<Control>,
}

impl State {
    pub const fn new() -> Self {
        Self {
            control: MaybeUninit::uninit(),
        }
    }
}

struct Control {
    interface: InterfaceNumber,
}

impl Handler for Control {
    fn control_out(&mut self, request: Request, _data: &[u8]) -> Option<OutResponse> {
        (request.request_type == RequestType::Class
            && request.recipient == Recipient::Interface
            && request.request == 34
            && request.index == u8::from(self.interface) as u16)
            .then_some(OutResponse::Accepted)
    }
}

pub struct LinkUsbClass<'d, D: Driver<'d>> {
    read: D::EndpointOut,
    write: D::EndpointIn,
}

impl<'d, D: Driver<'d>> LinkUsbClass<'d, D> {
    pub fn new(builder: &mut Builder<'d, D>, state: &'d mut State) -> Self {
        builder.msos_descriptor(0x0603_0000, 0x20);
        let mut function = builder.function(0xff, 0, 0);
        function.msos_feature(CompatibleIdFeatureDescriptor::new("WINUSB", ""));
        function.msos_feature(RegistryPropertyFeatureDescriptor::new(
            "DeviceInterfaceGUIDs",
            PropertyData::RegMultiSz(&["{F6C8A4DE-9A20-4F10-B6B6-46A5657D3767}"]),
        ));
        let mut interface = function.interface();
        let mut alternate = interface.alt_setting(0xff, 0, 0, None);
        let interface_number = alternate.interface_number();
        let read = alternate.endpoint_bulk_out(None, PACKET_SIZE as u16);
        let write = alternate.endpoint_bulk_in(None, PACKET_SIZE as u16);
        drop(function);
        builder.handler(state.control.write(Control {
            interface: interface_number,
        }));
        Self { read, write }
    }

    pub async fn run(mut self, store: &'static BondStore) -> ! {
        let mut request = [0_u8; MAX_FRAME_BYTES];
        loop {
            self.read.wait_enabled().await;
            loop {
                let length = match self.read_frame(&mut request).await {
                    Ok(length) => length,
                    Err(EndpointError::Disabled) => break,
                    Err(EndpointError::BufferOverflow) => continue,
                };
                let Some(response) = store.handle_link_frame(&request[..length]) else {
                    continue;
                };
                if self.write_frame(response.as_slice()).await.is_err() {
                    break;
                }
            }
        }
    }

    async fn read_frame(&mut self, buffer: &mut [u8]) -> Result<usize, EndpointError> {
        let mut length = 0;
        loop {
            let read = self.read.read(&mut buffer[length..]).await?;
            length += read;
            if read < PACKET_SIZE {
                return Ok(length);
            }
            if length == buffer.len() {
                return Err(EndpointError::BufferOverflow);
            }
        }
    }

    async fn write_frame(&mut self, frame: &[u8]) -> Result<(), EndpointError> {
        for chunk in frame.chunks(PACKET_SIZE) {
            self.write.write(chunk).await?;
        }
        if frame.len().is_multiple_of(PACKET_SIZE) {
            self.write.write(&[]).await?;
        }
        Ok(())
    }
}
