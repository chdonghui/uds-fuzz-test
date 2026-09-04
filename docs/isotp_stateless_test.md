# Stateless Docker ISO-TP 통신 테스트

Rust stateless vECU와 ISO-TP 테스트 도구를 Docker 컨테이너로 실행하여 UDS 통신을 확인하는 방법입니다.

## 통신 구성

```text
isotp-stateless-test 컨테이너                     vecu 컨테이너

TX 0x7E0  -- 22 F1 90 ------------->  RX 0x7E0
RX 0x7E8  <- 62 F1 90 01 02 03 04 --  TX 0x7E8
```

- CAN 인터페이스: `vcan0`
- 요청 CAN ID: `0x7E0`
- 응답 CAN ID: `0x7E8`
- 요청: `22 F1 90` (`ReadDataByIdentifier`)
- 예상 응답: `62 F1 90 01 02 03 04`

### Rust 라이브러리와 통신하는 과정

두 프로그램이 서로 직접 연결되는 것이 아니라, Linux 커널의 SocketCAN과 `vcan0`를 통해 통신합니다.

```text
isotpsend (can-utils)
        │  UDS 요청: 22 F1 90
        ▼
Linux ISO-TP 소켓 → vcan0 → Linux ISO-TP 소켓
                              ▼
                    socketcan-isotp (Rust)
                              ▼
                    handle_request()
                              │  UDS 응답: 62 F1 90 01 02 03 04
                              ▼
isotprecv (can-utils) ← vcan0 ← socketcan-isotp (Rust)
```

Rust 코드는 `socketcan-isotp` 라이브러리로 ISO-TP 소켓을 엽니다. 개념적으로 다음 세 값을 지정합니다.

```text
IsoTpSocket::open(vcan0, RX 0x7E0, TX 0x7E8)
```

- `0x7E0`: Rust ECU가 요청을 받는 ID
- `0x7E8`: Rust ECU가 응답을 보내는 ID

통신 순서:

1. `isotpsend`가 `0x7E0`으로 요청을 보냅니다.
2. Rust의 `socket.read()`가 완성된 ISO-TP 요청을 읽습니다.
3. `handle_request()`가 UDS 응답을 만듭니다.
4. `socket.write()`가 응답을 `0x7E8`로 보냅니다.
5. `isotprecv`가 응답을 받아 예상값과 비교합니다.

ISO-TP 메시지의 CAN 프레임 분할과 조립은 Linux 커널이 처리합니다. Rust 코드는 완성된 UDS 바이트만 읽고 씁니다.

## 요구 사항

이 테스트는 Linux SocketCAN을 사용하므로 Linux에서 실행해야 합니다.

필요한 항목:

- Docker
- Docker Compose 플러그인
- `iproute2`
- Linux `vcan` 커널 모듈
- `sudo` 권한

설치 여부는 다음 명령으로 확인할 수 있습니다.

```bash
docker --version
docker compose version
ip -Version
```

## 자동 실행 방법

프로젝트 루트에서 다음 스크립트를 실행합니다.

```bash
./scripts/run_isotp_stateless_test.sh
```

스크립트가 다음 작업을 자동으로 수행합니다.

1. `vcan` 커널 모듈 로드
2. `vcan0` 생성 및 활성화
3. `vecu`와 `isotp-stateless-test` Docker 이미지 빌드
4. Rust vECU 컨테이너 실행
5. ISO-TP 테스트 컨테이너 실행
6. UDS 요청 전송 및 응답 비교
7. 테스트 종료 후 컨테이너 정리

성공하면 다음 결과가 출력됩니다.

```text
PASS: 62 F1 90 01 02 03 04
```

응답이 다르거나 5초 안에 도착하지 않으면 테스트가 실패 코드로 종료됩니다.

## 수동 실행 방법

문제가 발생했을 때 각 단계를 직접 실행하고 로그를 확인하는 방법입니다.

### 1. `vcan0` 준비

```bash
sudo modprobe vcan
```

`vcan0`가 없다면 생성합니다.

```bash
sudo ip link add dev vcan0 type vcan
```

인터페이스를 활성화합니다.

```bash
sudo ip link set vcan0 up
```

상태를 확인합니다.

```bash
ip link show vcan0
```

### 2. Docker 이미지 빌드

```bash
docker compose -f docker/isotp-stateless.yml build
```

다음 이미지 두 개가 빌드됩니다.

- `vecu`: Rust vECU
- `isotp-stateless-test`: `can-utils` 기반 테스트 도구

### 3. vECU 실행

```bash
docker compose -f docker/isotp-stateless.yml up -d vecu
```

vECU 로그를 확인합니다.

```bash
docker compose -f docker/isotp-stateless.yml logs -f vecu
```

정상적으로 시작되면 다음과 비슷한 로그가 표시됩니다.

```text
Stateless UDS ECU listening on vcan0 (RX=0x7E0, TX=0x7E8)
```

로그 화면은 `Ctrl+C`로 빠져나올 수 있습니다. 백그라운드의 vECU 컨테이너는 계속 실행됩니다.

### 4. ISO-TP 테스트 실행

다른 터미널에서 프로젝트 루트로 이동한 후 실행합니다.

```bash
docker compose -f docker/isotp-stateless.yml run --rm isotp-stateless-test
```

정상 결과:

```text
PASS: 62 F1 90 01 02 03 04
```

vECU 로그에는 요청과 응답이 표시됩니다.

```text
RX [22, F1, 90]
TX [62, F1, 90, 01, 02, 03, 04]
```

### 5. 컨테이너 종료

```bash
docker compose -f docker/isotp-stateless.yml down
```

이 명령은 컨테이너만 정리하며 Docker 이미지와 `vcan0`는 유지합니다.

## 전체 정리 방법

테스트 컨테이너, 로컬 이미지, 볼륨, `vcan0`까지 정리하려면 다음 스크립트를 실행합니다.

```bash
./scripts/cleanup_isotp_stateless_test.sh
```

스크립트가 다음 항목을 정리합니다.

- `vecu`, `isotp-stateless-test` 컨테이너
- 테스트용 Docker 이미지
- Compose 볼륨과 고아 컨테이너
- 호스트의 `vcan0`
- 사용하지 않는 `vcan` 커널 모듈

## 관련 파일

| 파일 | 역할 |
|---|---|
| `docker/isotp-stateless.yml` | vECU와 테스트 컨테이너 정의 |
| `scripts/run_isotp_stateless_test.sh` | 전체 통신 테스트 자동 실행 |
| `scripts/cleanup_isotp_stateless_test.sh` | Docker 및 `vcan0` 전체 정리 |
| `ecu-projects/ecu-demo/rust/Dockerfile.isotp` | Rust vECU example 이미지 정의 |
| `ecu-projects/ecu-demo/rust/examples/isotp_stateless_ecu.rs` | Stateless UDS ECU 예제 |
| `tests/integration/isotp_stateless/Dockerfile` | ISO-TP 테스트 이미지 정의 |
| `tests/integration/isotp_stateless/test_uds.sh` | 요청 전송 및 응답 검사 |

## 주의 사항

macOS와 Windows의 Docker Desktop은 Linux VM 내부에서 컨테이너를 실행합니다. 호스트의 `vcan0`를 현재 구성과 같은 방식으로 공유할 수 없으므로 Linux 호스트 또는 Linux VM에서 실행해야 합니다.
