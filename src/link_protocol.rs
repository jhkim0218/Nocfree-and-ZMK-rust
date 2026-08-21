use crate::link_keymap::{
    LINK_COLS, LINK_LAYERS, LINK_ROWS, LinkBinding, LinkKeymap, raw_from_matrix,
};

pub const MAX_FRAME_BYTES: usize = 261;

const START: [u8; 2] = [0xff, 0xfe];
const END: [u8; 2] = [0xfe, 0xff];
const OK: u8 = 0;
const BAD_ARG: u8 = 1;
const OOR: u8 = 2;

const SET_KEY: u8 = 17;
const GET_KEY: u8 = 18;
const GET_LAYER_ROW: u8 = 20;
const SET_LAYER_ROW: u8 = 21;
const CLEAR_ALL: u8 = 33;
const CLEAR_LAYER: u8 = 34;
const READ_VERSION: u8 = 81;
const SET_SYSTEM: u8 = 84;
const READ_SYSTEM: u8 = 85;
const READ_BATTERY: u8 = 86;

const RSP_SET_KEY: u8 = 144;
const RSP_GET_KEY: u8 = 145;
const RSP_GET_LAYER_ROW: u8 = 147;
const RSP_SET_LAYER_ROW: u8 = 148;
const RSP_CLEAR_ALL: u8 = 161;
const RSP_CLEAR_LAYER: u8 = 162;
const RSP_READ_VERSION: u8 = 209;
const RSP_SET_SYSTEM: u8 = 212;
const RSP_READ_SYSTEM: u8 = 213;
const RSP_READ_BATTERY: u8 = 214;

pub struct LinkResponse {
    pub bytes: [u8; MAX_FRAME_BYTES],
    pub len: usize,
    pub changed: bool,
}

impl LinkResponse {
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

pub fn handle_request(frame: &[u8], keymap: &mut LinkKeymap) -> Option<LinkResponse> {
    let (op, payload) = decode_frame(frame)?;
    let mut response_payload = [0_u8; u8::MAX as usize];
    let mut payload_len = 1;
    let mut changed = false;

    let response_op = match op {
        SET_KEY => {
            if payload.len() != 6 {
                response_payload[0] = BAD_ARG;
            } else {
                let layer = payload[0] as usize;
                let row = payload[1] as usize;
                let col = payload[2] as usize;
                let binding =
                    LinkBinding::new(payload[3], u16::from_be_bytes([payload[4], payload[5]]));
                response_payload[0] = if !binding_is_valid(binding) {
                    BAD_ARG
                } else if keymap.set_matrix_binding(layer, row, col, binding) {
                    changed = true;
                    OK
                } else {
                    OOR
                };
            }
            RSP_SET_KEY
        }
        GET_KEY => {
            if payload.len() != 3 {
                response_payload[0] = BAD_ARG;
            } else {
                let layer = payload[0] as usize;
                let row = payload[1] as usize;
                let col = payload[2] as usize;
                if let Some(binding) = keymap.matrix_binding(layer, row, col) {
                    response_payload[..7].copy_from_slice(&[
                        OK,
                        payload[0],
                        payload[1],
                        payload[2],
                        binding.kind,
                        (binding.value >> 8) as u8,
                        binding.value as u8,
                    ]);
                    payload_len = 7;
                } else {
                    response_payload[0] = OOR;
                }
            }
            RSP_GET_KEY
        }
        GET_LAYER_ROW => {
            if payload.len() != 2
                || payload[0] as usize >= LINK_LAYERS
                || payload[1] as usize >= LINK_ROWS
            {
                response_payload[0] = OOR;
            } else {
                response_payload[0] = OK;
                response_payload[1] = payload[0];
                payload_len = 2;
                for col in 0..LINK_COLS {
                    let binding = keymap
                        .matrix_binding(payload[0] as usize, payload[1] as usize, col)
                        .unwrap_or(LinkBinding::TRANSPARENT);
                    response_payload[payload_len..payload_len + 4].copy_from_slice(&[
                        col as u8,
                        binding.kind,
                        (binding.value >> 8) as u8,
                        binding.value as u8,
                    ]);
                    payload_len += 4;
                }
            }
            RSP_GET_LAYER_ROW
        }
        SET_LAYER_ROW => {
            response_payload[0] = handle_set_layer_row(payload, keymap, &mut changed);
            RSP_SET_LAYER_ROW
        }
        CLEAR_ALL => {
            if payload.is_empty() {
                keymap.reset_all();
                response_payload[0] = OK;
                changed = true;
            } else {
                response_payload[0] = BAD_ARG;
            }
            RSP_CLEAR_ALL
        }
        CLEAR_LAYER => {
            if payload.len() == 1 && keymap.reset_layer(payload[0] as usize) {
                response_payload[0] = OK;
                changed = true;
            } else {
                response_payload[0] = OOR;
            }
            RSP_CLEAR_LAYER
        }
        READ_VERSION => {
            response_payload[..4].copy_from_slice(&[OK, 2, 3, 0]);
            payload_len = 4;
            RSP_READ_VERSION
        }
        SET_SYSTEM => {
            if payload.len() == 1 && keymap.set_system(payload[0]) {
                response_payload[0] = OK;
                changed = true;
            } else {
                response_payload[0] = BAD_ARG;
            }
            RSP_SET_SYSTEM
        }
        READ_SYSTEM => {
            response_payload[..2].copy_from_slice(&[OK, keymap.system()]);
            payload_len = 2;
            RSP_READ_SYSTEM
        }
        READ_BATTERY => {
            response_payload[..4].copy_from_slice(&[OK, 0xff, 0xff, 0xff]);
            payload_len = 4;
            RSP_READ_BATTERY
        }
        _ => return None,
    };

    let mut bytes = [0_u8; MAX_FRAME_BYTES];
    let len = encode_frame(response_op, &response_payload[..payload_len], &mut bytes)?;
    Some(LinkResponse {
        bytes,
        len,
        changed,
    })
}

fn handle_set_layer_row(payload: &[u8], keymap: &mut LinkKeymap, changed: &mut bool) -> u8 {
    if payload.len() < 2 || !(payload.len() - 2).is_multiple_of(4) {
        return BAD_ARG;
    }
    let layer = payload[0] as usize;
    let row = payload[1] as usize;
    if layer >= LINK_LAYERS || row >= LINK_ROWS {
        return OOR;
    }
    for chunk in payload[2..].chunks_exact(4) {
        let col = chunk[0] as usize;
        let binding = LinkBinding::new(chunk[1], u16::from_be_bytes([chunk[2], chunk[3]]));
        if raw_from_matrix(row, col).is_none() || !binding_is_valid(binding) {
            return BAD_ARG;
        }
    }
    for chunk in payload[2..].chunks_exact(4) {
        let binding = LinkBinding::new(chunk[1], u16::from_be_bytes([chunk[2], chunk[3]]));
        let _ = keymap.set_matrix_binding(layer, row, chunk[0] as usize, binding);
    }
    *changed = true;
    OK
}

fn binding_is_valid(binding: LinkBinding) -> bool {
    match binding.kind {
        0 => binding.value <= 0xff,
        1 => (0xe0..=0xe7).contains(&binding.value),
        2 => binding.value <= 0x03ff,
        3 | 0x80 => true,
        _ => false,
    }
}

fn decode_frame(frame: &[u8]) -> Option<(u8, &[u8])> {
    if frame.len() < 6 || frame[..2] != START || frame[frame.len() - 2..] != END {
        return None;
    }
    let payload_len = frame[3] as usize;
    (frame.len() == payload_len + 6).then_some((frame[2], &frame[4..4 + payload_len]))
}

fn encode_frame(op: u8, payload: &[u8], output: &mut [u8]) -> Option<usize> {
    if payload.len() > u8::MAX as usize || output.len() < payload.len() + 6 {
        return None;
    }
    output[..2].copy_from_slice(&START);
    output[2] = op;
    output[3] = payload.len() as u8;
    output[4..4 + payload.len()].copy_from_slice(payload);
    output[4 + payload.len()..6 + payload.len()].copy_from_slice(&END);
    Some(payload.len() + 6)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(op: u8, payload: &[u8]) -> [u8; MAX_FRAME_BYTES] {
        let mut frame = [0; MAX_FRAME_BYTES];
        let len = encode_frame(op, payload, &mut frame).unwrap();
        frame[len..].fill(0);
        frame[3] = payload.len() as u8;
        frame
    }

    fn run(op: u8, payload: &[u8], keymap: &mut LinkKeymap) -> LinkResponse {
        let frame = request(op, payload);
        handle_request(&frame[..payload.len() + 6], keymap).unwrap()
    }

    #[test]
    fn set_then_get_key_uses_nocfree_wire_format() {
        let mut keymap = LinkKeymap::default();
        let set = run(SET_KEY, &[2, 3, 7, 2, 0, 174], &mut keymap);
        assert_eq!(
            set.as_slice(),
            &[0xff, 0xfe, RSP_SET_KEY, 1, OK, 0xfe, 0xff]
        );
        assert!(set.changed);
        let get = run(GET_KEY, &[2, 3, 7], &mut keymap);
        assert_eq!(
            get.as_slice(),
            &[
                0xff,
                0xfe,
                RSP_GET_KEY,
                7,
                OK,
                2,
                3,
                7,
                2,
                0,
                174,
                0xfe,
                0xff
            ]
        );
    }

    #[test]
    fn layer_row_contains_all_twenty_one_columns() {
        let mut keymap = LinkKeymap::default();
        let response = run(GET_LAYER_ROW, &[0, 0], &mut keymap);
        assert_eq!(response.as_slice()[2], RSP_GET_LAYER_ROW);
        assert_eq!(response.as_slice()[3], 2 + LINK_COLS as u8 * 4);
        assert_eq!(response.len, 92);
        assert_eq!(&response.as_slice()[4..6], &[OK, 0]);
    }

    #[test]
    fn version_system_and_battery_match_link_expectations() {
        let mut keymap = LinkKeymap::default();
        assert_eq!(
            run(READ_VERSION, &[], &mut keymap).as_slice(),
            &[0xff, 0xfe, RSP_READ_VERSION, 4, OK, 2, 3, 0, 0xfe, 0xff]
        );
        assert_eq!(run(READ_SYSTEM, &[], &mut keymap).as_slice()[4..6], [OK, 1]);
        assert_eq!(
            run(READ_BATTERY, &[], &mut keymap).as_slice()[4..8],
            [OK, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn malformed_and_out_of_range_requests_fail_loudly() {
        let mut keymap = LinkKeymap::default();
        assert!(handle_request(&[0xff, 0xfe, GET_KEY, 0, 0, 0], &mut keymap).is_none());
        let response = run(SET_KEY, &[0, 9, 9, 0, 0, 4], &mut keymap);
        assert_eq!(response.as_slice()[4], OOR);
        assert!(!response.changed);
    }
}
