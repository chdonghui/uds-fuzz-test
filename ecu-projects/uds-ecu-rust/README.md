# uds-ecu-rust

`vcan0`에서 ISO-TP 요청을 처리하는 간단한 stateful UDS ECU입니다.

- 요청 CAN ID: `0x7E0`
- 응답 CAN ID: `0x7E8`
- 시작 세션: Default Session (`0x01`)
- DID `0xF190`: Extended Session (`0x03`)에서만 허용

## 구조

```text
src/lib.rs                          # ecu 모듈 공개
src/main.rs                         # ISO-TP 연결과 세션 보관
src/ecu.rs                          # UDS 요청 처리
examples/replay_stateful_simple.rs  # corpus 파일 하나 Replay
examples/replay_stateful_summary.rs # corpus 폴더의 단순 통계
examples/replay_stateful.rs         # 목표 Stateful 시퀀스 분석
fuzz/fuzz_targets/fuzz_target_1.rs  # 단일 요청 퍼징
fuzz/fuzz_targets/stateful-sequence.rs # 상태 전이 퍼징
```

## cargo-fuzz

호스트에 nightly Rust와 `cargo-fuzz`를 설치한 후 `uds-ecu-rust` 폴더에서 실행합니다.

단일 요청 퍼징:

```bash
cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=60
```

Stateful 퍼징:

```bash
cargo +nightly fuzz run stateful-sequence -- -max_total_time=60
```

## Stateful corpus Replay

`uds-ecu-rust` 폴더에서 실행합니다.

파일 하나의 요청과 응답 확인:

```bash
cargo run --example replay_stateful_simple -- \
  fuzz/corpus/stateful-sequence/<corpus-file>
```

폴더 전체의 파일, 요청과 응답 개수 확인:

```bash
cargo run --example replay_stateful_summary -- \
  fuzz/corpus/stateful-sequence
```

목표 UDS 시퀀스까지 분석:

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence
```

자세한 내용:

- [cargo-fuzz 사용법](../../docs/cargo_fuzz.md)
- [Stateful 퍼징](../../docs/cargo_stateful_fuzz.md)
- [Stateful 결과 확인](../../docs/cargo_stateful_fuzz_results.md)

## Docker 자동 통신 테스트

프로젝트 루트에서 실행합니다.

```bash
./scripts/run_uds_ecu_rust_test.sh
```

테스트는 같은 ECU 프로세스에 세션 변경, DID 읽기, Level 1 Seed 요청과 Key 전송을 순서대로 보내 상태가 유지되는지 확인합니다.

```text
PASS session: 50 03
PASS DID: 62 F1 90 01 02 03 04
PASS seed: 67 01 12 34
PASS key: 67 02
```

전체 정리:

```bash
./scripts/cleanup_uds_ecu_rust_test.sh
```

## 직접 실행

Linux에서 `vcan0`를 준비합니다.

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
```

`vcan0`가 이미 존재하면 `ip link add`는 생략합니다.

프로젝트 루트에서 실행합니다.

```bash
cargo run --manifest-path ecu-projects/uds-ecu-rust/Cargo.toml
```

Extended Session으로 전환한 뒤 DID를 읽습니다.

```text
10 03    -> 50 03
22 F1 90    -> 62 F1 90 01 02 03 04
27 01       -> 67 01 12 34
27 02 B8 9E -> 67 02
```
