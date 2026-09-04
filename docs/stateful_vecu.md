# Stateful vECU

Stateful vECU는 이전 요청으로 변경된 ECU 상태를 기억하고, 현재 상태에 따라 다른 응답을 반환합니다.

이 예제에서는 `session` 변수가 현재 진단 세션을 저장합니다.

```text
0x01: Default Session
0x02: Programming Session
0x03: Extended Diagnostic Session
0x04: Safety System Diagnostic Session
```

## Stateless와 차이

Stateless ECU는 같은 요청에 항상 같은 응답을 반환합니다.

```text
22 F1 90 -> 항상 같은 응답
```

Stateful ECU는 이전에 받은 세션 변경 요청에 따라 응답이 달라집니다.

```text
Default Session에서 22 F1 90  -> 거부
10 03으로 Extended 전환
Extended Session에서 22 F1 90 -> 허용
```

## 예제 구성

| 파일 | 역할 |
|---|---|
| `examples/stateful_ecu_mvp.rs` | 세션 상태 변경의 최소 구현 |
| `examples/stateful_ecu.rs` | 세션, 요청 길이, DID, NRC 처리 보강 |
| `examples/isotp_stateful.rs` | `vcan0`에서 ISO-TP 요청을 수신하고 응답 |
| `examples/isotp_stateful_ecu.rs` | ISO-TP 코드에서 분리한 UDS 처리 로직 |

## 1. Stateful MVP

파일: `ecu-projects/ecu-demo/rust/examples/stateful_ecu_mvp.rs`

세션은 처음에 Default Session으로 시작합니다.

```rust
let mut session: u8 = 0x01;
```

`handle_request()`는 `&mut u8`로 세션을 전달받습니다.

```rust
fn handle_request(session: &mut u8, req: &[u8]) -> Vec<u8>
```

`&mut`는 함수가 기존 `session` 값을 읽고 변경할 수 있다는 뜻입니다.

```rust
[0x10, 0x03] => {
    *session = 0x03;
    vec![0x50, 0x03]
}
```

`10 03` 요청을 받으면 세션을 Extended Session으로 바꾸고 `50 03`을 응답합니다.

MVP의 핵심 흐름:

```text
현재 세션  요청       응답        변경 후 세션
Default    22 F1 90   7F 22 7F   Default
Default    10 03      50 03      Extended
Extended   22 F1 90   62 F1 90 01 Extended
Extended   10 01      50 01      Default
Default    22 F1 90   7F 22 7F   Default
```

## 2. Stateful ECU

파일: `ecu-projects/ecu-demo/rust/examples/stateful_ecu.rs`

MVP에서 다음 처리를 추가한 예제입니다.

- 네 가지 Diagnostic Session Control sub-function
- 지원하지 않는 세션 검사
- 잘못된 요청 길이 검사
- Extended Session에서만 DID `0xF190` 허용
- 지원하지 않는 DID 검사
- 지원하지 않는 SID 검사

### DiagnosticSessionControl

요청 SID는 `0x10`, 긍정 응답 SID는 `0x50`입니다.

```text
10 01 -> 50 01  Default Session
10 02 -> 50 02  Programming Session
10 03 -> 50 03  Extended Diagnostic Session
10 04 -> 50 04  Safety System Diagnostic Session
```

지원하지 않는 sub-function:

```text
10 05 -> 7F 10 12
```

잘못된 길이:

```text
10 -> 7F 10 13
```

### ReadDataByIdentifier

DID `0xF190`은 학습을 위해 Extended Session에서만 읽을 수 있게 구현되어 있습니다.

```rust
[0x22, 0xF1, 0x90] if *session == 0x03 => {
    vec![0x62, 0xF1, 0x90, 0x01]
}
```

`if *session == 0x03`은 현재 세션이 Extended인지 검사하는 match guard입니다.

Default Session에서 요청:

```text
22 F1 90 -> 7F 22 7F
```

Extended Session으로 전환한 후 요청:

```text
10 03    -> 50 03
22 F1 90 -> 62 F1 90 01
```

Extended Session에서 지원하지 않는 DID 요청:

```text
22 12 34 -> 7F 22 31
```

잘못된 길이:

```text
22 F1 -> 7F 22 13
```

## NRC 정리

Negative Response는 다음 형식입니다.

```text
7F 요청_SID NRC
```

| NRC | 이름 | 예제에서의 의미 |
|---|---|---|
| `0x11` | ServiceNotSupported | 지원하지 않는 SID |
| `0x12` | SubFunctionNotSupported | 지원하지 않는 세션 sub-function |
| `0x13` | IncorrectMessageLengthOrInvalidFormat | 잘못된 요청 길이 |
| `0x31` | RequestOutOfRange | 지원하지 않는 DID |
| `0x7F` | ServiceNotSupportedInActiveSession | 현재 세션에서 서비스를 사용할 수 없음 |

예를 들어 `7F 22 7F`에서 첫 번째 `7F`는 Negative Response SID이고, 마지막 `7F`는 NRC입니다.

## 3. ISO-TP 연결

파일: `ecu-projects/ecu-demo/rust/examples/isotp_stateful.rs`

`socketcan-isotp`로 `vcan0`에 연결합니다.

```rust
let mut socket = IsoTpSocket::open(
    "vcan0",
    StandardId::new(0x7E0).expect("invalid RX ID"),
    StandardId::new(0x7E8).expect("invalid TX ID"),
)?;
```

- RX `0x7E0`: 테스터가 보낸 요청 수신
- TX `0x7E8`: ECU 응답 송신

세션 변수는 반복문 밖에서 한 번 생성됩니다.

```rust
let mut session: u8 = 0x01;

loop {
    let req = socket.read()?;
    let response = handle_request(&mut session, &req);
    socket.write(&response)?;
}
```

반복문이 실행되는 동안 같은 `session` 변수를 계속 사용하므로 이전 요청에서 변경한 세션이 다음 요청에도 유지됩니다.

## 4. ISO-TP와 UDS 로직 분리

`isotp_stateful.rs`는 다음 선언으로 UDS 로직을 가져옵니다.

```rust
#[path = "isotp_stateful_ecu.rs"]
mod isotp_stateful_ecu;
use isotp_stateful_ecu::handle_request;
```

역할은 다음과 같이 나뉩니다.

```text
isotp_stateful.rs
├── vcan0 연결
├── ISO-TP 요청 수신
├── session 보관
└── ISO-TP 응답 송신

isotp_stateful_ecu.rs
└── session과 UDS 요청을 검사하여 응답 생성
```

통신 계층과 UDS 처리 로직을 분리하면 각 코드의 역할을 이해하고 수정하기 쉽습니다.

## 실행 방법

### Stateful 로직만 실행

```bash
cargo run \
    --manifest-path ecu-projects/ecu-demo/rust/Cargo.toml \
    --example stateful_ecu
```

이 실행은 미리 정의된 요청 배열을 처리하므로 `vcan0`가 필요하지 않습니다.

### ISO-TP Stateful ECU 실행

먼저 Linux에서 `vcan0`를 준비합니다.

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
```

`vcan0`가 이미 존재하면 `ip link add`는 생략합니다.

ECU를 실행합니다.

```bash
cargo run \
    --manifest-path ecu-projects/ecu-demo/rust/Cargo.toml \
    --example isotp_stateful
```

이 예제는 `vcan0`에서 요청을 기다리며, 실행 중에는 세션 상태를 계속 유지합니다.
