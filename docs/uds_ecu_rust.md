# uds-ecu-rust

`ecu-projects/uds-ecu-rust`는 `vcan0`에서 ISO-TP 요청을 처리하는 Stateful Rust UDS ECU입니다.

학습용 example과 달리 실제 ECU 기능을 계속 추가하기 위한 독립 Cargo 프로젝트입니다.

## 현재 구조

```text
ecu-projects/uds-ecu-rust/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── README.md
├── src/
│   ├── lib.rs
│   ├── main.rs
│   └── ecu.rs
├── examples/
│   ├── replay_stateful_simple.rs
│   ├── replay_stateful_summary.rs
│   └── replay_stateful.rs
└── fuzz/
    ├── Cargo.toml
    └── fuzz_targets/
        ├── fuzz_target_1.rs
        └── stateful-sequence.rs
```

주요 파일 역할:

- `src/lib.rs`: `ecu` 모듈을 실행 프로그램과 퍼징 하네스에 공개
- `src/main.rs`: ISO-TP 연결, 요청 수신, 응답 송신과 ECU 상태 보관
- `src/ecu.rs`: ECU 상태 정의와 UDS 요청 처리
- `examples/replay_stateful_simple.rs`: corpus 파일 하나의 요청과 응답 확인
- `examples/replay_stateful_summary.rs`: Stateful corpus 폴더의 단순 통계 확인
- `examples/replay_stateful.rs`: Positive Response와 목표 시퀀스 분석
- `fuzz/fuzz_targets/fuzz_target_1.rs`: 단일 UDS 요청 퍼징
- `fuzz/fuzz_targets/stateful-sequence.rs`: 여러 UDS 요청의 상태 전이 퍼징
- `Cargo.toml`: Rust 패키지와 `socketcan-isotp` 의존성 정의
- `Dockerfile`: `uds-ecu-rust` Docker 이미지 빌드 및 실행

## 코드 연결 구조

```text
src/main.rs
├── vcan0 ISO-TP 소켓 생성
├── EcuState 생성
├── socket.read()
├── handle_request(&mut state, &req)
└── socket.write()

src/ecu.rs
├── EcuState
└── handle_request()
```

`main.rs`가 실제 통신을 담당하고 `ecu.rs`가 UDS 요청을 해석합니다.

## ISO-TP 설정

```text
인터페이스: vcan0
요청 RX ID: 0x7E0
응답 TX ID: 0x7E8
```

Rust에서는 다음과 같이 소켓을 엽니다.

```rust
let mut socket = IsoTpSocket::open(
    "vcan0",
    StandardId::new(0x7E0).expect("invalid RX ID"),
    StandardId::new(0x7E8).expect("invalid TX ID"),
)?;
```

통신 흐름:

```text
Tester                  uds-ecu-rust

요청, ID 0x7E0  ──────> socket.read()
                         handle_request()
응답, ID 0x7E8  <────── socket.write()
```

## ECU 상태

`EcuState`는 요청 사이에 유지해야 하는 값을 저장합니다.

```rust
pub struct EcuState {
    pub session: u8,
    pub seed: Option<u16>,
    pub security_unlocked: bool,
}
```

| 필드 | 의미 | 초기값 |
|---|---|---|
| `session` | 현재 Diagnostic Session | `0x01` |
| `seed` | SecurityAccess에서 발급한 Seed | `None` |
| `security_unlocked` | 보안 잠금 해제 여부 | `false` |

`session`은 서비스 접근 조건 확인에 사용됩니다. `seed`와 `security_unlocked`는 `0x27 SecurityAccess`의 Seed/Key 순서와 잠금 해제 상태를 저장합니다.

## 현재 지원 서비스

### `0x10 DiagnosticSessionControl`

```text
10 01 -> 50 01  Default Session
10 03 -> 50 03  Extended Diagnostic Session
```

지원하지 않는 sub-function:

```text
10 05 -> 7F 10 12
```

잘못된 길이:

```text
10 -> 7F 10 13
```

### `0x22 ReadDataByIdentifier`

DID `0xF190`은 Extended Session에서만 허용됩니다.

Default Session:

```text
22 F1 90 -> 7F 22 7F
```

Extended Session:

```text
10 03    -> 50 03
22 F1 90 -> 62 F1 90 01 02 03 04
```

지원하지 않는 DID:

```text
22 12 34 -> 7F 22 31
```

잘못된 길이:

```text
22 F1 -> 7F 22 13
```

### `0x27 SecurityAccess`

Level 1 Seed/Key 흐름을 학습용으로 구현했습니다. Extended Session에서 Seed `0x1234`를 요청하고, XOR로 계산한 Key `0xB89E`를 전송합니다.

```text
10 03          -> 50 03
27 01          -> 67 01 12 34
27 02 B8 9E    -> 67 02
```

Key 계산:

```text
0x1234 XOR 0xAAAA = 0xB89E
```

Seed 요청 없이 Key를 보내면 NRC `0x24`, 잘못된 Key는 NRC `0x35`를 반환합니다. XOR 계산과 고정 Seed는 학습용이며 실제 보안에 사용하면 안 됩니다.

현재 `security_unlocked` 상태 저장까지 구현됐지만, 이 상태로 보호하는 별도 서비스나 DID는 아직 없습니다.

`Some(seed)`, `state.seed`, `from_be_bytes()` 등 구현에 사용된 Rust 문법은 [UDS 코드에서 사용하는 Rust 문법](rust_uds_syntax.md)을 참고하세요.

## Cargo로 직접 실행

Linux에서 `vcan0`를 준비합니다.

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
```

이미 `vcan0`가 존재하면 `ip link add`는 생략합니다.

프로젝트 루트에서 ECU를 실행합니다.

```bash
cargo run --manifest-path ecu-projects/uds-ecu-rust/Cargo.toml
```

정상적으로 실행되면 요청을 기다립니다.

```text
waiting...
```

컴파일만 확인하려면:

```bash
cargo check --manifest-path ecu-projects/uds-ecu-rust/Cargo.toml
```

## Docker 자동 통신 테스트

프로젝트 루트에서 실행합니다.

```bash
./scripts/run_uds_ecu_rust_test.sh
```

스크립트가 다음 작업을 수행합니다.

1. `vcan0` 준비
2. `uds-ecu-rust`와 테스트 이미지 빌드
3. `uds-ecu-rust` 컨테이너 실행
4. `10 03` 요청과 `50 03` 응답 확인
5. `22 F1 90` 요청과 DID 응답 확인
6. `27 01` Seed 요청과 응답 확인
7. `27 02 B8 9E` Key 검증 응답 확인
8. 컨테이너 정리

정상 결과:

```text
PASS session: 50 03
PASS DID: 62 F1 90 01 02 03 04
PASS seed: 67 01 12 34
PASS key: 67 02
```

모든 요청은 같은 ECU 프로세스에 전달됩니다. 따라서 Extended Session과 발급된 Seed 상태가 다음 요청까지 유지됩니다.

## 수동 Docker 테스트

이미지 빌드:

```bash
docker compose -f docker/uds-ecu-rust.yml build
```

ECU 실행:

```bash
docker compose -f docker/uds-ecu-rust.yml up -d uds-ecu-rust
```

테스트 실행:

```bash
docker compose -f docker/uds-ecu-rust.yml run --rm uds-ecu-rust-test
```

ECU 로그 확인:

```bash
docker compose -f docker/uds-ecu-rust.yml logs uds-ecu-rust
```

컨테이너 종료:

```bash
docker compose -f docker/uds-ecu-rust.yml down
```

## 전체 정리

컨테이너, 테스트 이미지와 `vcan0`를 정리합니다.

```bash
./scripts/cleanup_uds_ecu_rust_test.sh
```

## 관련 파일

| 파일 | 역할 |
|---|---|
| `docker/uds-ecu-rust.yml` | ECU와 테스트 컨테이너 정의 |
| `scripts/run_uds_ecu_rust_test.sh` | 자동 통신 테스트 실행 |
| `scripts/cleanup_uds_ecu_rust_test.sh` | 테스트 환경 전체 정리 |
| `tests/integration/uds_ecu_rust/Dockerfile` | `can-utils` 테스트 이미지 정의 |
| `tests/integration/uds_ecu_rust/test_uds.sh` | 세션 변경과 DID 응답 검사 |

Docker의 공통 구조는 [Docker 구성과 사용법](docker.md), Stateful UDS 개념은 [Stateful vECU](stateful_vecu.md), 코드 문법은 [UDS 코드에서 사용하는 Rust 문법](rust_uds_syntax.md)을 참고하세요.

`lib.rs`의 역할과 `handle_request()`를 퍼징하는 절차는 [uds-ecu-rust cargo-fuzz 사용법](cargo_fuzz.md)을 참고하세요.
