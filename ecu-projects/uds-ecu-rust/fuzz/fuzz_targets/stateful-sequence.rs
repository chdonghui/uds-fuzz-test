#![no_main]

use libfuzzer_sys::fuzz_target;
use uds_ecu_rust::ecu::{EcuState, handle_request};

const SEND_REQUEST: u8 = 0x01;
const RESET: u8 = 0x04;

fuzz_target!(|data: &[u8]| {
    let mut state = EcuState::new();
    let mut offset = 0;

    while offset + 3 <= data.len() {
        let action = data[offset];
        let length = u16::from_be_bytes([data[offset + 1], data[offset + 2]]) as usize;
        offset += 3;

        if length > data.len() - offset {
            break;
        }

        let payload = &data[offset..offset + length];
        offset += length;

        match action {
            SEND_REQUEST => {
                let _ = handle_request(&mut state, payload);
            }
            RESET => {
                state = EcuState::new();
            }
            _ => {}
        }
    }
});
