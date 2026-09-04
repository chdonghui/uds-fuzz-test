# Stateless vECU

Stateless vECU는 이전 요청을 기억하지 않고, 현재 들어온 요청만 보고 응답을 만듭니다.

```text
요청 → handle_request() → 응답
```

세션이나 이전 요청을 저장하는 변수가 없으므로 같은 요청에는 항상 같은 응답을 반환합니다.

## 예제 구성

| 파일 | 역할 |
|---|---|
| `examples/stateless_ecu_mvp.rs` | Stateless UDS 처리의 최소 구현 |
| `examples/stateless_ecu.rs` | 요청 길이, DID, 세션 sub-function, NRC 처리 보강 |
| `examples/isotp_stateless_ecu.rs` | `vcan0`에서 ISO-TP 요청을 받고 UDS 응답 전송 |

## 1. Stateless MVP

파일: `ecu-projects/ecu-demo/rust/examples/stateless_ecu_mvp.rs`

`handle_request()`는 UDS 요청 바이트를 받아 응답 바이트를 반환합니다.

```rust
fn handle_request(req: &[u8]) -> Vec<u8> {
    match req {
        [0x22, 0xF1, 0x90] => vec![0x62, 0xF1, 0x90, 0x01],
        [0x22, ..] => vec![0x7F, 0x22, 0x31],
        [sid, ..] => vec![0x7F, *sid, 0x11],
        [] => vec![],
    }
}
```

요청은 위에서 아래 순서로 비교됩니다.

1. `22 F1 90`이면 정상 응답
2. `0x22`로 시작하지만 정상 요청이 아니면 NRC `0x31`
3. 다른 SID이면 NRC `0x11`
4. 빈 요청이면 빈 응답

### ReadDataByIdentifier

`0x22`는 ReadDataByIdentifier 서비스입니다. ECU 내부 데이터를 DID로 조회합니다.

```text
22 F1 90
│  └──── DID 0xF190
└─────── SID 0x22
```

정상 응답:

```text
22 F1 90 -> 62 F1 90 01
```

긍정 응답 SID는 요청 SID에 `0x40`을 더합니다.

```text
0x22 + 0x40 = 0x62
```

## 2. Stateless ECU

파일: `ecu-projects/ecu-demo/rust/examples/stateless_ecu.rs`

MVP에서 다음 처리를 추가한 예제입니다.

- 지원하지 않는 DID 검사
- 잘못된 요청 길이 검사
- DiagnosticSessionControl 응답
- 지원하지 않는 sub-function 검사
- 지원하지 않는 SID 검사

### ReadDataByIdentifier 처리

```rust
[0x22, 0xF1, 0x90] => vec![0x62, 0xF1, 0x90, 0x01],
[0x22, _, _] => vec![0x7F, 0x22, 0x31],
[0x22, ..] => vec![0x7F, 0x22, 0x13],
```

요청과 응답:

```text
22 F1 90 -> 62 F1 90 01   지원 DID
22 12 34 -> 7F 22 31      지원하지 않는 DID
22 F1    -> 7F 22 13      잘못된 길이
```

`[0x22, _, _]`는 정확히 3바이트인 `0x22` 요청을 의미합니다. 따라서 지원 DID가 아니면 NRC `0x31`을 반환합니다.

`[0x22, ..]`는 그 외 길이의 `0x22` 요청과 일치하며 NRC `0x13`을 반환합니다.

### DiagnosticSessionControl 처리

요청 SID는 `0x10`, 긍정 응답 SID는 `0x50`입니다.

```text
10 01 -> 50 01
10 02 -> 50 02
10 03 -> 50 03
10 04 -> 50 04
```

지원하지 않는 sub-function과 잘못된 길이:

```text
10 05 -> 7F 10 12
10    -> 7F 10 13
```

이 예제는 `0x10` 요청에 응답하지만 세션 값을 저장하지 않습니다. 따라서 이름처럼 stateless이며, 이후 요청의 동작은 바뀌지 않습니다.

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

## 3. ISO-TP Stateless ECU

파일: `ecu-projects/ecu-demo/rust/examples/isotp_stateless_ecu.rs`

`socketcan-isotp` 라이브러리로 `vcan0`에 연결합니다.

```rust
let mut socket = IsoTpSocket::open(
    "vcan0",
    StandardId::new(0x7E0).expect("invalid RX ID"),
    StandardId::new(0x7E8).expect("invalid TX ID"),
)?;
```

- RX `0x7E0`: 테스터 요청 수신
- TX `0x7E8`: ECU 응답 송신

요청 처리 반복문:

```rust
loop {
    let req = socket.read()?;
    let response = handle_request(&req);
    socket.write(&response)?;
}
```

통신 순서:

```text
Tester                Rust vECU
  │                       │
  │  22 F1 90 (0x7E0)     │
  ├──────────────────────>│
  │                       │ handle_request()
  │  62 F1 90 01 02 03 04 │
  │<──────────────────────┤ (0x7E8)
```

ISO-TP의 CAN 프레임 분할과 조립은 Linux 커널이 처리합니다. Rust 코드는 완성된 UDS 요청을 읽고 완성된 UDS 응답을 씁니다.

### 시작 시 응답 확인

ISO-TP 소켓을 열기 전에 `assert_eq!`로 대표 응답을 확인합니다.

```rust
assert_eq!(
    handle_request(&[0x22, 0xF1, 0x90]),
    vec![0x62, 0xF1, 0x90, 0x01, 0x02, 0x03, 0x04]
);
```

실제 결과가 예상값과 다르면 프로그램이 종료됩니다. 이 검사는 UDS 함수만 확인하며 `vcan0` 통신은 확인하지 않습니다.

## 실행 방법

### MVP 실행

```bash
cargo run \
    --manifest-path ecu-projects/ecu-demo/rust/Cargo.toml \
    --example stateless_ecu_mvp
```

### Stateless ECU 로직 실행

```bash
cargo run \
    --manifest-path ecu-projects/ecu-demo/rust/Cargo.toml \
    --example stateless_ecu
```

두 실행은 미리 정의된 요청 배열을 처리하므로 `vcan0`가 필요하지 않습니다.

### ISO-TP 자동 통신 테스트

Linux에서 다음 스크립트를 실행합니다.

```bash
./scripts/run_isotp_stateless_test.sh
```

스크립트가 `vcan0` 준비, Docker 이미지 빌드, vECU 실행, 요청 전송, 응답 검사를 자동으로 수행합니다.

성공 결과:

```text
PASS: 62 F1 90 01 02 03 04
```

자세한 Docker 통신 과정은 [Stateless ISO-TP 통합 테스트](isotp_stateless_test.md)를 참고하세요.
