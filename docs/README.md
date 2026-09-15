# 문서 목차

UDS, Rust vECU, ISO-TP 통신, 퍼징을 학습하기 위한 문서 모음입니다.

## 추천 학습 순서

처음 프로젝트를 살펴본다면 다음 순서로 읽는 것을 권장합니다.

1. [프로젝트 시작 방법](instructions.md)
2. [Docker 구성과 사용법](docker.md)
3. [Rust 컴파일](rust_compile.md)
4. [Stateless vECU](stateless_vecu.md)
5. [Stateful vECU](stateful_vecu.md)
6. [uds-ecu-rust 구조와 실행 방법](uds_ecu_rust.md)
7. [UDS 코드에서 사용하는 Rust 문법](rust_uds_syntax.md)
8. [uds-ecu-rust cargo-fuzz 사용법](cargo_fuzz.md)
9. [Stateful 퍼징](cargo_stateful_fuzz.md)
10. [Stateful 퍼징 결과 확인](cargo_stateful_fuzz_results.md)
11. [Stateful Replay 코드 이해](replay_stateful_code.md)

## 시작하기

### [프로젝트 시작 방법](instructions.md)

저장소 초기화, Docker 실행, ISO-TP 자동 테스트, `uds-ecu-rust` 직접 실행 방법을 설명합니다.

### [Docker 구성과 사용법](docker.md)

Dockerfile, Compose 파일, 컨테이너 연결 구조와 테스트 스크립트의 역할을 설명합니다.

## Rust

### [Rust 컴파일](rust_compile.md)

Cargo를 이용해 Rust 예제를 빌드하고 실행하는 기본 방법을 설명합니다.

### [UDS 코드에서 사용하는 Rust 문법](rust_uds_syntax.md)

`EcuState`, `impl`, `Self`, `Option`, `Some`, slice 패턴, Big Endian 변환 등 `uds-ecu-rust` 코드에 사용된 문법을 설명합니다.

## vECU와 UDS

### [Stateless vECU](stateless_vecu.md)

요청마다 이전 상태를 유지하지 않는 단순한 UDS ECU의 구조를 설명합니다.

### [Stateful vECU](stateful_vecu.md)

Diagnostic Session과 SecurityAccess처럼 요청 사이에 상태를 유지하는 ECU의 개념을 설명합니다.

### [uds-ecu-rust 구조와 실행 방법](uds_ecu_rust.md)

실제 개발 중인 Stateful Rust ECU의 파일 구조, 지원 서비스, Cargo 실행 및 Docker 통신 테스트 방법을 설명합니다.

## ISO-TP 통신 테스트

### [Stateless ISO-TP 통합 테스트](isotp_stateless_test.md)

두 Docker 컨테이너와 `vcan0`를 사용해 Stateless Rust ECU의 요청·응답을 확인하는 과정을 설명합니다.

`uds-ecu-rust`의 Stateful 자동 통신 테스트는 [uds-ecu-rust 구조와 실행 방법](uds_ecu_rust.md#docker-자동-통신-테스트)을 참고하세요.

## 퍼징

### [libFuzzer 사용법](libFuzzer.md)

C 하네스와 libFuzzer 기본 예제, `iso14229` stateless 서버 퍼저의 Docker 자동 실행 방법을 설명합니다.

### [uds-ecu-rust cargo-fuzz 사용법](cargo_fuzz.md)

Rust에서 `lib.rs`로 UDS 로직을 공개하는 이유, 퍼징 하네스 작성법, `cargo-fuzz` 실행 및 오류 입력 재현 방법을 설명합니다.

### [Stateful 퍼징](cargo_stateful_fuzz.md)

하나의 ECU 상태에 여러 UDS 요청을 순서대로 전달하는 하네스와 입력 형식을 설명합니다.

### [Stateful 퍼징 결과 확인](cargo_stateful_fuzz_results.md)

Stateful corpus를 Replay하여 요청 순서와 실제 UDS 응답을 확인하는 원리와 사용 방식을 설명합니다.

### [Stateful Replay 코드 이해](replay_stateful_code.md)

`replay_stateful.rs`의 함수 호출 흐름, corpus 파싱 과정과 코드에 사용된 Rust 문법을 설명합니다.

### [cargo-fuzz 결과 해석](cargo_fuzz_results.md)

cargo-fuzz의 터미널 로그, corpus, artifacts와 코드 커버리지를 확인하는 일반적인 방법을 설명합니다.

## 계획

### [향후 계획](future.md)

완료된 퍼징 단계와 Boundary, Stateful, Coverage 및 upstream 제보 계획을 정리합니다.

## 목적별 빠른 찾기

- 프로젝트를 처음 실행하려면 [프로젝트 시작 방법](instructions.md)을 참고하세요.
- Docker 파일의 역할을 이해하려면 [Docker 구성과 사용법](docker.md)을 참고하세요.
- Rust 예제를 실행하려면 [Rust 컴파일](rust_compile.md)을 참고하세요.
- Stateless와 Stateful의 차이는 [Stateless vECU](stateless_vecu.md)와 [Stateful vECU](stateful_vecu.md)를 참고하세요.
- `uds-ecu-rust`를 실행하고 테스트하려면 [uds-ecu-rust 구조와 실행 방법](uds_ecu_rust.md)을 참고하세요.
- `EcuState`와 `Option` 문법은 [UDS 코드에서 사용하는 Rust 문법](rust_uds_syntax.md)을 참고하세요.
- Docker로 ISO-TP 통신을 확인하려면 [Stateless ISO-TP 통합 테스트](isotp_stateless_test.md)를 참고하세요.
- C 코드를 퍼징하려면 [libFuzzer 사용법](libFuzzer.md)을 참고하세요.
- Rust `handle_request()`를 퍼징하려면 [uds-ecu-rust cargo-fuzz 사용법](cargo_fuzz.md)을 참고하세요.
- 여러 요청의 상태 전이를 퍼징하려면 [Stateful 퍼징](cargo_stateful_fuzz.md)을 참고하세요.
- Stateful corpus의 요청과 응답을 확인하려면 [Stateful 퍼징 결과 확인](cargo_stateful_fuzz_results.md)을 참고하세요.
- Replay 예제 코드를 이해하려면 [Stateful Replay 코드 이해](replay_stateful_code.md)를 참고하세요.
- 일반적인 퍼징 결과를 확인하려면 [cargo-fuzz 결과 해석](cargo_fuzz_results.md)을 참고하세요.
- 현재 퍼징 진행 상태와 다음 작업은 [퍼징 진행 상태와 계획](future.md)을 참고하세요.
