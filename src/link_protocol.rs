use crate::link_keymap::{
    HOTKEY_SLOTS, HotkeySlot, LINK_COLS, LINK_LAYERS, LINK_ROWS, LinkBinding, LinkKeymap,
    raw_from_matrix,
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
const SET_TEXT: u8 = 48;
const GET_TEXT: u8 = 49;
const CLEAR_TEXT: u8 = 50;
const DELETE_TEXT: u8 = 51;
const SET_HOTKEY: u8 = 52;
const GET_HOTKEY: u8 = 53;
const CLEAR_HOTKEY: u8 = 54;
const DELETE_HOTKEY: u8 = 55;
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
const RSP_SET_TEXT: u8 = 176;
const RSP_GET_TEXT: u8 = 177;
const RSP_CLEAR_TEXT: u8 = 178;
const RSP_DELETE_TEXT: u8 = 179;
const RSP_SET_HOTKEY: u8 = 180;
const RSP_GET_HOTKEY: u8 = 181;
const RSP_CLEAR_HOTKEY: u8 = 182;
const RSP_DELETE_HOTKEY: u8 = 183;
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
        SET_HOTKEY => {
            response_payload[0] = handle_set_hotkey(payload, keymap, &mut changed);
            RSP_SET_HOTKEY
        }
        GET_HOTKEY => {
            payload_len = encode_hotkey(payload, keymap, &mut response_payload);
            RSP_GET_HOTKEY
        }
        CLEAR_HOTKEY | DELETE_HOTKEY => {
            response_payload[0] = handle_clear_hotkey(payload, keymap, &mut changed);
            if op == CLEAR_HOTKEY {
                RSP_CLEAR_HOTKEY
            } else {
                RSP_DELETE_HOTKEY
            }
        }
        GET_TEXT => {
            if payload.len() == 1 && (payload[0] as usize) < HOTKEY_SLOTS {
                response_payload[..3].copy_from_slice(&[OK, payload[0], 0]);
                payload_len = 3;
            } else {
                response_payload[0] = OOR;
            }
            RSP_GET_TEXT
        }
        SET_TEXT | CLEAR_TEXT | DELETE_TEXT => {
            response_payload[0] = BAD_ARG;
            match op {
                SET_TEXT => RSP_SET_TEXT,
                CLEAR_TEXT => RSP_CLEAR_TEXT,
                _ => RSP_DELETE_TEXT,
            }
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

fn handle_set_hotkey(payload: &[u8], keymap: &mut LinkKeymap, changed: &mut bool) -> u8 {
    if payload.len() < 6 || payload[0] as usize >= HOTKEY_SLOTS {
        return BAD_ARG;
    }
    if payload[1] == 0 {
        return handle_clear_hotkey(&payload[..1], keymap, changed);
    }
    let event_count = payload[5] as usize;
    if payload.len() != 6 + event_count * 3 || event_count == 0 || event_count > 5 {
        return BAD_ARG;
    }
    let unbound = payload[2..5] == [u8::MAX; 3];
    if !unbound
        && (payload[4] as usize >= LINK_LAYERS
            || raw_from_matrix(payload[2] as usize, payload[3] as usize).is_none())
    {
        return OOR;
    }

    let mut modifiers = 0_u8;
    let mut key = None;
    for event in payload[6..].chunks_exact(3) {
        if event[2] != 1 {
            return BAD_ARG;
        }
        let usage = u16::from_be_bytes([event[0], event[1]]);
        match usage {
            0xe0..=0xe7 => modifiers |= 1 << (usage - 0xe0),
            0x04..=0xff if key.is_none() => key = Some(usage),
            _ => return BAD_ARG,
        }
    }
    let Some(key) = key else {
        return BAD_ARG;
    };
    let hotkey = HotkeySlot {
        empty: false,
        row: payload[2],
        col: payload[3],
        layer: payload[4],
        modifiers,
        key,
    };
    let _ = keymap.set_hotkey(payload[0] as usize, hotkey);
    *changed = true;
    OK
}

fn encode_hotkey(payload: &[u8], keymap: &LinkKeymap, output: &mut [u8]) -> usize {
    if payload.len() != 1 || payload[0] as usize >= HOTKEY_SLOTS {
        output[0] = OOR;
        return 1;
    }
    let slot_id = payload[0];
    let slot = keymap.hotkey(slot_id as usize).unwrap_or(HotkeySlot::EMPTY);
    if slot.empty {
        output[..3].copy_from_slice(&[OK, slot_id, 0]);
        return 3;
    }
    output[..7].copy_from_slice(&[OK, slot_id, 1, slot.row, slot.col, slot.layer, 0]);
    let mut length = 7;
    for bit in 0..8 {
        if slot.modifiers & (1 << bit) != 0 {
            output[length..length + 3].copy_from_slice(&[0, 0xe0 + bit, 1]);
            length += 3;
            output[6] += 1;
        }
    }
    output[length..length + 3].copy_from_slice(&[(slot.key >> 8) as u8, slot.key as u8, 1]);
    output[6] += 1;
    length + 3
}

fn handle_clear_hotkey(payload: &[u8], keymap: &mut LinkKeymap, changed: &mut bool) -> u8 {
    if payload.len() != 1 || !keymap.clear_hotkey(payload[0] as usize) {
        return OOR;
    }
    *changed = true;
    OK
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

    #[test]
    fn hotkey_slots_round_trip_in_link_wire_format() {
        let mut keymap = LinkKeymap::default();
        let set_payload = [
            2,
            1,
            u8::MAX,
            u8::MAX,
            u8::MAX,
            3,
            0,
            0xe0,
            1,
            0,
            0xe2,
            1,
            0,
            0x4c,
            1,
        ];
        let set = run(SET_HOTKEY, &set_payload, &mut keymap);
        assert_eq!(set.as_slice()[2], RSP_SET_HOTKEY);
        assert_eq!(set.as_slice()[4], OK);
        assert!(set.changed);

        let get = run(GET_HOTKEY, &[2], &mut keymap);
        assert_eq!(get.as_slice()[2], RSP_GET_HOTKEY);
        assert_eq!(
            &get.as_slice()[4..],
            &[
                OK,
                2,
                1,
                u8::MAX,
                u8::MAX,
                u8::MAX,
                3,
                0,
                0xe0,
                1,
                0,
                0xe2,
                1,
                0,
                0x4c,
                1,
                0xfe,
                0xff,
            ]
        );
        assert_eq!(
            keymap.hotkey(2).unwrap().action(),
            crate::keymap::Action::Chord {
                modifiers: (1 << 0) | (1 << 2),
                key: 0x4c,
            }
        );
    }

    #[test]
    fn quick_text_queries_return_empty_without_timing_out_link() {
        let mut keymap = LinkKeymap::default();
        let get = run(GET_TEXT, &[15], &mut keymap);
        assert_eq!(
            get.as_slice(),
            &[0xff, 0xfe, RSP_GET_TEXT, 3, OK, 15, 0, 0xfe, 0xff]
        );
        let unsupported = run(SET_TEXT, &[0, 1, 0xff, 0xff, 0xff, 1, b'a'], &mut keymap);
        assert_eq!(unsupported.as_slice()[4], BAD_ARG);
        assert!(!unsupported.changed);
    }
}
