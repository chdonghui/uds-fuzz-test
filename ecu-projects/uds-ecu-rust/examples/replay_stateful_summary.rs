use std::{env, fs};

use uds_ecu_rust::ecu::{EcuState, handle_request};

const SEND_REQUEST: u8 = 0x01;
const RESET: u8 = 0x04;

// 파일 하나에서 센 결과입니다.
#[derive(Default)]
struct FileResult {
    requests: usize,
    positive_responses: usize,
    negative_responses: usize,
    no_responses: usize,
}

// corpus 파일 하나를 실행하고 요청과 응답의 개수를 셉니다.
fn replay_file(data: &[u8]) -> FileResult {
    let mut result = FileResult::default();
    let mut state = EcuState::new();
    let mut offset = 0;

    while offset + 3 <= data.len() {
        let action = data[offset];
        let length = u16::from_be_bytes([data[offset + 1], data[offset + 2]]) as usize;
        offset += 3;

        // 선언된 길이만큼 Payload가 없으면 현재 파일 처리를 끝냅니다.
        if length > data.len() - offset {
            break;
        }

        let payload = &data[offset..offset + length];
        offset += length;

        match action {
            SEND_REQUEST => {
                let response = handle_request(&mut state, payload);
                result.requests += 1;

                // UDS Negative Response는 0x7F로 시작합니다.
                if response.is_empty() {
                    result.no_responses += 1;
                } else if response.first() == Some(&0x7F) {
                    result.negative_responses += 1;
                } else {
                    result.positive_responses += 1;
                }
            }
            RESET => {
                state = EcuState::new();
            }
            _ => {
                // 알 수 없는 Action은 퍼징 하네스와 동일하게 무시합니다.
            }
        }
    }

    result
}

fn main() {
    // 첫 번째 명령행 인수로 corpus 폴더 경로를 받습니다.
    let directory = env::args()
        .nth(1)
        .expect("Usage: replay_stateful_summary <corpus-directory>");

    let entries = fs::read_dir(directory).expect("failed to read corpus directory");

    let mut file_count = 0;
    let mut request_count = 0;
    let mut positive_count = 0;
    let mut negative_count = 0;
    let mut no_response_count = 0;

    // 폴더 안의 corpus 파일을 하나씩 읽어 결과를 더합니다.
    for entry in entries {
        let path = entry.expect("failed to read directory entry").path();

        if !path.is_file() {
            continue;
        }

        let data = fs::read(path).expect("failed to read corpus file");
        let result = replay_file(&data);

        file_count += 1;
        request_count += result.requests;
        positive_count += result.positive_responses;
        negative_count += result.negative_responses;
        no_response_count += result.no_responses;
    }

    println!("Files: {file_count}");
    println!("SEND_REQUEST actions: {request_count}");
    println!("Positive responses: {positive_count}");
    println!("Negative responses: {negative_count}");
    println!("No response: {no_response_count}");
}
