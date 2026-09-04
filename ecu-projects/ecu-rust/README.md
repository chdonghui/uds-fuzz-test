# ecu-rust

`vcan0`에서 ISO-TP 요청을 처리하는 간단한 stateful UDS ECU입니다.

- 요청 CAN ID: `0x7E0`
- 응답 CAN ID: `0x7E8`
- 시작 세션: Default Session (`0x01`)
- DID `0xF190`: Extended Session (`0x03`)에서만 허용

## 구조

```text
src/main.rs  # ISO-TP 연결과 세션 보관
src/ecu.rs   # UDS 요청 처리
```

## 실행

Linux에서 `vcan0`를 준비합니다.

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
```

`vcan0`가 이미 존재하면 `ip link add`는 생략합니다.

프로젝트 루트에서 실행합니다.

```bash
cargo run --manifest-path ecu-projects/ecu-rust/Cargo.toml
```

Extended Session으로 전환한 뒤 DID를 읽습니다.

```text
10 03    -> 50 03
22 F1 90 -> 62 F1 90 01 02 03 04
```
