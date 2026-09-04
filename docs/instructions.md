# 프로젝트 시작 방법

프로젝트를 처음 실행할 때 필요한 기본 절차입니다. 상세 설명은 각 주제별 문서를 참고하세요.

## 1. Git submodule 초기화

이 프로젝트는 `third_party/iso14229`를 Git submodule로 사용합니다.

저장소를 clone한 후 프로젝트 루트에서 실행합니다.

```bash
git submodule update --init --recursive
```

상태 확인:

```bash
git submodule status
```

## 2. 필요한 도구 확인

기본 개발 환경에는 Docker와 Docker Compose가 필요합니다.

```bash
docker --version
docker compose version
```

ISO-TP 테스트에는 Linux와 `iproute2`도 필요합니다.

```bash
ip -Version
```

## 3. 퍼징 개발 컨테이너 실행

```bash
./docker_run.sh
```

이 스크립트는 `uds-fuzz` 컨테이너를 실행하고 Bash로 접속합니다. 프로젝트 폴더는 컨테이너의 `/work`에 연결됩니다.

종료:

```bash
exit
docker compose down
```

자세한 Docker 구성은 [Docker 구성과 사용법](docker.md)을 참고하세요.

## 4. Stateless ISO-TP 통신 테스트

Linux에서 다음 스크립트를 실행합니다.

```bash
./scripts/run_isotp_stateless_test.sh
```

정상 결과:

```text
PASS: 62 F1 90 01 02 03 04
```

전체 정리:

```bash
./scripts/cleanup_isotp_stateless_test.sh
```

상세 통신 과정은 [Stateless ISO-TP 통합 테스트](isotp_stateless_test.md)를 참고하세요.

## 5. Rust ECU 자동 통신 테스트

```bash
./scripts/run_uds_ecu_rust_test.sh
```

정상 결과:

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

## 6. Stateful Rust ECU 직접 실행

먼저 Linux에서 `vcan0`를 준비합니다.

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
```

`vcan0`가 이미 존재하면 `ip link add`는 생략합니다.

ECU 실행:

```bash
cargo run --manifest-path ecu-projects/uds-ecu-rust/Cargo.toml
```

## 관련 문서

- [Docker 구성과 사용법](docker.md)
- [libFuzzer 사용법](libFuzzer.md)
- [Rust 컴파일](rust_compile.md)
- [Stateless vECU](stateless_vecu.md)
- [Stateful vECU](stateful_vecu.md)
- [uds-ecu-rust 구조와 실행 방법](uds_ecu_rust.md)
- [UDS 코드에서 사용하는 Rust 문법](rust_uds_syntax.md)
- [uds-ecu-rust cargo-fuzz 사용법](cargo_fuzz.md)
- [Stateless ISO-TP 통합 테스트](isotp_stateless_test.md)
