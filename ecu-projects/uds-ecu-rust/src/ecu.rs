pub struct EcuState {
    pub session: u8,
    pub seed: Option<u16>,
    pub security_unlocked: bool,
}

impl EcuState {
    pub fn new() -> Self {
        Self {
            session: 0x01,
            seed: None,
            security_unlocked: false,
        }
    }
}

fn calculate_key(seed: u16) -> u16 {
    seed ^ 0xAAAA
}

pub fn handle_request(state: &mut EcuState, req: &[u8]) -> Vec<u8> {
    match req {
        [0x10, 0x01] => {
            state.session = 0x01;
            state.seed = None;
            state.security_unlocked = false;
            vec![0x50, 0x01]
        }
        [0x10, 0x03] => {
            state.session = 0x03;
            state.seed = None;
            state.security_unlocked = false;
            vec![0x50, 0x03]
        }
        [0x10, _] => vec![0x7F, 0x10, 0x12],
        [0x10, ..] => vec![0x7F, 0x10, 0x13],

        [0x22, 0xF1, 0x90] if state.session == 0x03 => {
            vec![0x62, 0xF1, 0x90, 0x01, 0x02, 0x03, 0x04]
        }
        [0x22, _, _] if state.session != 0x03 => vec![0x7F, 0x22, 0x7F],
        [0x22, _, _] => vec![0x7F, 0x22, 0x31],
        [0x22, ..] => vec![0x7F, 0x22, 0x13],

        [0x27, 0x01] if state.session != 0x03 => {
            vec![0x7F, 0x27, 0x22]
        }

        [0x27, 0x01] => {
            let seed: u16 = 0x1234; // 2byte seed 만들기
            state.seed = Some(seed); // 나중에 확인 가능하도록 seed를 ECU 상태에 저장

            let [high, low] = seed.to_be_bytes(); // u16 값을 두 바이트로 나눔

            vec![0x27 + 0x40, 0x01, high, low] // 응답을 만듦
        } // seed 요청

        [0x27, 0x02, key_high, key_low] => {
            let Some(seed) = state.seed else {
                return vec![0x7F, 0x27, 0x24];
            };

            let received_key = u16::from_be_bytes([*key_high, *key_low]);
            let expected_key = calculate_key(seed);

            if received_key == expected_key {
                state.security_unlocked = true;
                state.seed = None;

                vec![0x67, 0x02]
            } else {
                vec![0x7F, 0x27, 0x35]
            }
        } // key 전송
        [0x27, 0x01, ..] | [0x27, 0x02, ..] => vec![0x7F, 0x27, 0x13],
        [0x27, _] => vec![0x7F, 0x27, 0x12],
        [0x27, ..] => vec![0x7F, 0x27, 0x13],

        [sid, ..] => vec![0x7F, *sid, 0x11],
        [] => vec![],
    }
}
