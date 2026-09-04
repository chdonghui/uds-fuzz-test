# uds-fuzz-test

UDS와 ISO-TP를 학습하고, Rust vECU 및 libFuzzer 기반 테스트를 실습하는 프로젝트입니다.

## 현재 구성

```text
docker/                # 추가 Docker Compose 구성
ecu-projects/
├── ecu-demo/          # Rust 기반 UDS/ISO-TP 학습 예제
└── uds-ecu-rust/      # Stateful Rust UDS ECU
fuzz/
├── examples/          # libFuzzer 동작 확인 예제
└── server_fuzz.c      # iso14229 UDS 서버 퍼징 하네스
tests/integration/     # Docker 기반 ISO-TP 통합 테스트
scripts/               # 실행 자동화 스크립트
third_party/iso14229/  # C 기반 UDS 라이브러리 서브모듈
```

## 문서

- [전체 문서 목차와 추천 학습 순서](docs/README.md)
- [기초 사용법](docs/instructions.md)
- [Docker 구성과 사용법](docs/docker.md)
- [libFuzzer 사용법](docs/libFuzzer.md)
- [Rust 컴파일](docs/rust_compile.md)
- [Stateless vECU 학습 내용](docs/stateless_vecu.md)
- [Stateful vECU 학습 내용](docs/stateful_vecu.md)
- [uds-ecu-rust 구조와 실행 방법](docs/uds_ecu_rust.md)
- [UDS 코드에서 사용하는 Rust 문법](docs/rust_uds_syntax.md)
- [uds-ecu-rust cargo-fuzz 사용법](docs/cargo_fuzz.md)
- [Stateful 퍼징](docs/cargo_stateful_fuzz.md)
- [Stateful 퍼징 결과 확인](docs/cargo_stateful_fuzz_results.md)
- [Stateful Replay 코드 이해](docs/replay_stateful_code.md)
- [cargo-fuzz 결과 해석](docs/cargo_fuzz_results.md)
- [Stateless ISO-TP 통합 테스트](docs/isotp_stateless_test.md)
- [향후 계획](docs/future.md)

## 주요 구성 요소

### `third_party/iso14229`

[driftregion/iso14229](https://github.com/driftregion/iso14229)를 Git submodule로 포함합니다. C로 작성된 UDS 프로토콜 라이브러리이며 서버 퍼징 대상입니다.

### `fuzz/examples/libfuzzer_basic.c`

`UDS` 입력에서 의도적인 크래시를 발생시켜 libFuzzer 동작을 확인하는 기초 예제입니다.

### `fuzz/server_fuzz.c`

libFuzzer 입력을 `iso14229`의 UDS 서버에 전달하는 퍼징 하네스입니다.

### `ecu-projects/uds-ecu-rust`

Stateful UDS 처리 로직과 ISO-TP 실행 코드를 분리한 Rust ECU 프로젝트입니다.

### `ecu-projects/ecu-demo/rust/examples/isotp_stateless_ecu.rs`

`socketcan-isotp`를 이용해 `vcan0`에서 UDS 요청을 수신하고 응답하는 stateless ECU 학습 예제입니다.

## Rust vECU 통신 테스트

Linux에서 다음 명령 하나로 Docker 컨테이너 두 개를 빌드하고 통신을 검사합니다.

```bash
./scripts/run_isotp_stateless_test.sh
```

실행되는 컨테이너:

- `vecu`: `isotp_stateless_ecu` Cargo example을 실행
- `isotp-stateless-test`: `22 F1 90` 요청을 보내고 응답 확인

정상 응답:

```text
22 F1 90 -> 62 F1 90 01 02 03 04
PASS: 62 F1 90 01 02 03 04
```

이 테스트는 Linux SocketCAN과 `vcan` 커널 모듈을 사용합니다. macOS와 Windows의 Docker Desktop에서는 동일한 방식으로 직접 실행할 수 없습니다.

테스트 이미지와 `vcan0`까지 모두 정리하려면 다음 명령을 실행합니다.

```bash
./scripts/cleanup_isotp_stateless_test.sh
```

자세한 내용은 [Stateless ISO-TP 통합 테스트](docs/isotp_stateless_test.md)를 참고하세요.

## Stateful Rust vECU 통신 테스트

Linux에서 다음 명령 하나로 `uds-ecu-rust`와 Tester 컨테이너를 실행하고 Stateful UDS 통신을 검사합니다.

```bash
./scripts/run_uds_ecu_rust_test.sh
```

확인하는 요청 순서:

```text
10 03          -> 50 03
22 F1 90       -> 62 F1 90 01 02 03 04
27 01          -> 67 01 12 34
27 02 B8 9E    -> 67 02
```

테스트 환경을 정리하려면 다음 명령을 실행합니다.

```bash
./scripts/cleanup_uds_ecu_rust_test.sh
```

자세한 내용은 [uds-ecu-rust 구조와 실행 방법](docs/uds_ecu_rust.md)을 참고하세요.

## Rust ECU 퍼징

```bash
cd ecu-projects/uds-ecu-rust
```

단일 요청 퍼징:

```bash
cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=60
```

Stateful 퍼징:

```bash
cargo +nightly fuzz run stateful-sequence -- -max_total_time=60
```

Stateful corpus 단순 요약:

```bash
cargo run --example replay_stateful_summary -- \
  fuzz/corpus/stateful-sequence
```

목표 UDS 시퀀스 분석:

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence
```

자세한 내용은 [cargo-fuzz 사용법](docs/cargo_fuzz.md)과 [Stateful 결과 확인](docs/cargo_stateful_fuzz_results.md)을 참고하세요.
