# Stateful 퍼징

하나의 ECU 상태에 여러 요청과 동작을 순서대로 전달하는 cargo-fuzz 방법입니다.

현재 Rust ECU뿐 아니라 `iso14229.c`와 다른 OSS의 하네스에도 적용할 수 있는 입력 구조를 목표로 합니다.

## 단일 요청 퍼징과 차이

단일 요청 하네스는 퍼징 입력 전체를 요청 하나로 전달합니다.

```rust
let mut state = EcuState::new();
let _ = handle_request(&mut state, data);
```

이 방식은 정상 요청 바이트를 발견하는 데 적합하지만 다음과 같은 상태 전이를 표현할 수 없습니다.

```text
10 03
→ Extended Session 진입
→ 22 F1 90
→ DID 처리
```

Stateful 하네스는 입력 하나를 여러 동작으로 나누고 같은 ECU 상태에 순서대로 적용합니다.

## 기존 단일 타깃 유지

단일 요청 타깃은 삭제하지 않고 Stateful 타깃을 추가합니다.

```bash
cargo +nightly fuzz add stateful-sequence
```

```text
fuzz/fuzz_targets/
├── fuzz_target_1.rs
└── stateful-sequence.rs
```

- `fuzz_target_1`: UDS payload 하나를 직접 전달
- `stateful-sequence`: 여러 요청과 상태 변화 전달

기존 단일 타깃 이름을 변경했다면 `fuzz_target_1.rs` 대신 해당 파일명이 표시됩니다.

## 범용 입력 구분 규칙

Stateful 입력을 다음 레코드의 반복으로 구성합니다.

```text
[Action: 1바이트][Length: 2바이트][Payload: Length바이트]
```

- `Action`: 하네스가 수행할 동작
- `Length`: Payload 길이, `u16` Big Endian
- `Payload`: 대상 OSS에 전달할 원본 데이터

최소 동작:

```text
0x01 SEND_REQUEST
0x02 POLL
0x03 ADVANCE_TIME
0x04 RESET
```

현재 Rust ECU에서는 우선 다음 두 동작만 처리해도 됩니다.

```text
0x01 SEND_REQUEST
0x04 RESET
```

다른 동작은 `iso14229.c`나 다른 OSS의 API에 맞춰 나중에 연결할 수 있습니다.

## UDS 규칙은 변경하지 않음

예를 들어 다음 레코드는 UDS 요청 하나를 나타냅니다.

```text
01 00 03 22 F1 90
```

해석:

```text
01          SEND_REQUEST
00 03       Payload 길이 3
22 F1 90    원본 UDS payload
```

하네스는 `Action`과 `Length`를 제거하고 대상 함수에는 원본 UDS payload만 전달합니다.

```rust
handle_request(&mut state, &[0x22, 0xF1, 0x90]);
```

따라서 `01 00 03`은 UDS나 ISO-TP 데이터가 아니라 하네스가 시나리오를 해석하기 위한 정보입니다.

```text
하네스가 해석: Action + Length
ECU가 해석:    Payload 원본
```

## 왜 길이는 2바이트인가?

1바이트 길이는 payload를 최대 255바이트로 제한합니다. `u16` 길이는 최대 65,535바이트까지 표현할 수 있어 긴 UDS payload를 사용하는 OSS에도 적용하기 쉽습니다.

Big Endian 길이 예:

```text
00 02 → 2바이트
00 03 → 3바이트
01 00 → 256바이트
```

## 요청 시퀀스 예

Extended Session 진입 후 DID를 요청하는 시나리오:

```text
01 00 02 10 03 01 00 03 22 F1 90
```

레코드별 해석:

```text
01 00 02 10 03       SEND_REQUEST, 10 03
01 00 03 22 F1 90    SEND_REQUEST, 22 F1 90
```

실행 흐름:

```text
EcuState::new()
    ↓
10 03 처리
    ↓
session = 0x03
    ↓
22 F1 90 처리
```

SecurityAccess 시나리오:

```text
01 00 02 10 03
01 00 02 27 01
01 00 04 27 02 B8 9E
```

한 줄로 표현하면:

```text
01 00 02 10 03 01 00 02 27 01 01 00 04 27 02 B8 9E
```

## Stateful 하네스

`fuzz/fuzz_targets/stateful-sequence.rs`:

```rust
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
        let length = u16::from_be_bytes([
            data[offset + 1],
            data[offset + 2],
        ]) as usize;
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
```

## 하네스 코드 해석

### 레코드 헤더 확인

```rust
while offset + 3 <= data.len()
```

`Action` 1바이트와 `Length` 2바이트를 읽을 수 있을 때만 다음 레코드를 처리합니다.

### 동작과 길이 읽기

```rust
let action = data[offset];
let length = u16::from_be_bytes([
    data[offset + 1],
    data[offset + 2],
]) as usize;
```

`from_be_bytes()`로 두 바이트 길이를 하나의 `u16` 값으로 합칩니다.

### 범위 확인

```rust
if length > data.len() - offset {
    break;
}
```

입력에 선언된 길이만큼 payload가 남아 있지 않으면 처리를 종료합니다. 이 확인은 범위를 벗어난 slice로 인한 panic을 막습니다.

### 원본 payload 분리

```rust
let payload = &data[offset..offset + length];
```

하네스 헤더를 제외한 payload만 가져옵니다.

### 동작 실행

```rust
match action {
    SEND_REQUEST => {
        let _ = handle_request(&mut state, payload);
    }
    RESET => {
        state = EcuState::new();
    }
    _ => {}
}
```

`SEND_REQUEST`는 원본 payload를 ECU에 전달하고, `RESET`은 ECU 상태를 초기화합니다.

## 다른 OSS에 연결

입력 레코드 형식은 유지하고 `Action`을 실제 OSS API에 연결합니다.

```text
동일한 입력 레코드
    ├── Rust ECU → handle_request()
    ├── iso14229.c → Mock transport와 UDSServerPoll()
    └── 다른 OSS → 해당 라이브러리의 request/poll/reset API
```

예를 들어 `iso14229.c`에서는 다음처럼 연결할 수 있습니다.

```text
SEND_REQUEST → Mock transport RX에 payload 저장
POLL         → UDSServerPoll() 호출
ADVANCE_TIME → Mock 시간 증가
RESET        → 서버와 transport 상태 초기화
```

OSS마다 함수 이름과 호출 순서는 다르지만 `Action + Length + Payload` 형식은 재사용할 수 있습니다.

## 퍼징 범위

이 하네스는 ISO-TP 처리가 끝난 UDS payload와 ECU 상태 처리를 대상으로 합니다.

```text
SEND_REQUEST의 Payload: 22 F1 90
ECU에 전달되는 요청:    22 F1 90
```

`Action`과 `Length`는 여러 UDS 요청과 API 동작을 구분하기 위한 하네스 정보이며 ECU에 전달하지 않습니다.

이 문서에서는 ISO-TP 프레임 파싱이나 재조립을 퍼징하지 않습니다.

## 실행 방법

```bash
cd ecu-projects/uds-ecu-rust
cargo +nightly fuzz run stateful-sequence -- -max_total_time=60
```

중지는 `Ctrl+C`입니다.

## corpus 확인

위치:

```text
fuzz/corpus/stateful-sequence/
```

단일 corpus 파일의 원본 바이트를 확인합니다.

```bash
xxd -p -u -c 4096 \
  fuzz/corpus/stateful-sequence/<corpus-file> \
  | sed 's/../& /g'
```

모든 corpus를 바이트 단위로 확인합니다.

```bash
for file in fuzz/corpus/stateful-sequence/*; do
    echo -n "$(basename "$file"): "
    xxd -p -u -c 4096 "$file" | sed 's/../& /g'
done
```

Stateful corpus는 각 레코드를 다음 순서로 반복해서 해석합니다.

```text
Action → Length → Payload → Action → Length → Payload
```

`xxd`는 corpus의 원본 바이트와 요청 순서를 보여주지만 ECU 응답은 보여주지 않습니다. 요청과 실제 응답을 함께 확인하려면 간단한 Replay 예제를 실행합니다.

```bash
cargo run --example replay_stateful_simple -- \
  fuzz/corpus/stateful-sequence/<corpus-file>
```

```text
xxd
→ 원본 Action, Length, Payload 확인

replay_stateful_simple
→ Payload를 ECU에 전달하고 요청과 응답 확인
```

## 결과 확인

- corpus에 의미 있는 요청과 동작 시퀀스가 있는가?
- 목표 상태로 변경된 뒤 다음 요청이 처리됐는가?
- artifacts에 크래시나 timeout 입력이 있는가?
- 코드 커버리지에서 목표 분기가 실행됐는가?

corpus에 시퀀스가 있다는 사실만으로 목표 응답 분기까지 실행됐다고 단정할 수는 없습니다. 정확한 실행 경로는 코드 커버리지로 확인해야 합니다.

일반적인 로그, corpus와 artifacts 해석 방법은 [cargo-fuzz 결과 해석](cargo_fuzz_results.md)을 참고하세요.

## 주의점

이 레코드 형식은 실제 UDS 또는 ISO-TP 패킷 형식이 아니라 여러 API 동작을 표현하는 하네스 입력 형식입니다. 대상 OSS에는 반드시 Payload 원본만 전달해야 합니다.

고정 Seed `0x1234`와 XOR Key `0xB89E`는 학습용이며 실제 SecurityAccess 보안 방식으로 사용하면 안 됩니다.
