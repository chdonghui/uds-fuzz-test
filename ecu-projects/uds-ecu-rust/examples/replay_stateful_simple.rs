use std::{env, fs};

use uds_ecu_rust::ecu::{EcuState, handle_request};

// Corpus 레코드 형식: [Action 1바이트][Length 2바이트][Payload]
const SEND_REQUEST: u8 = 0x01;
const RESET: u8 = 0x04;

fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn main() {
    // 첫 번째 명령행 인수로 corpus 파일 경로를 받습니다.
    let path = env::args()
        .nth(1)
        .expect("Usage: replay_stateful_simple <corpus-file>");
    let data = fs::read(&path).expect("failed to read corpus file");

    // 하나의 corpus 파일에 있는 요청들은 같은 ECU 상태를 공유합니다.
    let mut state = EcuState::new();
    let mut offset = 0;

    // Action 1바이트와 Length 2바이트를 읽을 수 있는 동안 반복합니다.
    while offset + 3 <= data.len() {
        let action = data[offset];
        let length = u16::from_be_bytes([data[offset + 1], data[offset + 2]]) as usize;
        offset += 3;

        // 선언된 길이보다 Payload가 짧으면 잘린 레코드이므로 종료합니다.
        if length > data.len() - offset {
            println!("TRUNCATED RECORD");
            break;
        }

        let payload = &data[offset..offset + length];
        offset += length;

        match action {
            SEND_REQUEST => {
                let response = handle_request(&mut state, payload);
                println!("Request:  {}", hex(payload));
                println!("Response: {}", hex(&response));
            }
            RESET => {
                state = EcuState::new();
                println!("RESET");
            }
            _ => {
                // 퍼징 하네스와 동일하게 알 수 없는 Action은 무시합니다.
            }
        }
    }
}
