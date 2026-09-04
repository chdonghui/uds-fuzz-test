#![no_main]

use libfuzzer_sys::fuzz_target;
use uds_ecu_rust::ecu::{EcuState, handle_request};

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
    let mut state = EcuState::new();
    let _ = handle_request(&mut state, data);
});
