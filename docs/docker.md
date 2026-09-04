# Docker 구성과 사용법

이 프로젝트는 두 가지 Docker 구성을 사용합니다.

1. 퍼징과 개발에 사용하는 `uds-fuzz` 컨테이너
2. Stateless ISO-TP 통신을 확인하는 `vecu`, `isotp-stateless-test` 컨테이너

## Dockerfile과 Compose 차이

```text
Dockerfile
└── 이미지에 무엇을 설치하고 어떤 명령을 실행할지 정의

Compose
└── 어떤 컨테이너를 실행하고 어떻게 연결할지 정의
```

Dockerfile은 컨테이너의 재료와 실행 환경을 만들고, Compose는 해당 이미지로 하나 이상의 컨테이너를 관리합니다.

## 전체 연결 구조

```text
기본 개발 환경

compose.yml
└── Dockerfile
    └── uds-fuzz 컨테이너
        └── 호스트 프로젝트를 /work에 연결

Stateless ISO-TP 테스트

docker/isotp-stateless.yml
├── ecu-projects/ecu-demo/rust/Dockerfile.isotp
│   └── vecu 컨테이너
│       └── isotp_stateless_ecu Cargo example 실행
│
└── tests/integration/isotp_stateless/Dockerfile
    └── isotp-stateless-test 컨테이너
        └── test_uds.sh 실행
```

## 파일별 역할

| 파일 | 역할 |
|---|---|
| `Dockerfile` | Clang과 빌드 도구가 설치된 퍼징 개발 이미지 |
| `compose.yml` | 기본 `uds-fuzz` 개발 컨테이너 정의 |
| `docker_run.sh` | 개발 컨테이너 실행 후 Bash 접속 |
| `docker/isotp-stateless.yml` | vECU와 ISO-TP 테스트 컨테이너 정의 |
| `ecu-projects/ecu-demo/rust/Dockerfile.isotp` | Rust stateless vECU 이미지 빌드 |
| `tests/integration/isotp_stateless/Dockerfile` | `can-utils` 설치 및 테스트 스크립트 실행 |
| `scripts/run_isotp_stateless_test.sh` | `vcan0` 준비부터 통신 확인까지 자동 실행 |
| `scripts/cleanup_isotp_stateless_test.sh` | 테스트 컨테이너, 이미지와 `vcan0` 정리 |
| `docker/uds-ecu-rust.yml` | 실제 `uds-ecu-rust`와 테스트 컨테이너 정의 |
| `ecu-projects/uds-ecu-rust/Dockerfile` | Stateful Rust ECU 이미지 빌드 |
| `tests/integration/uds_ecu_rust/Dockerfile` | `uds-ecu-rust` 통합 테스트 이미지 빌드 |
| `scripts/run_uds_ecu_rust_test.sh` | 세션 변경과 DID 요청 자동 테스트 |
| `scripts/cleanup_uds_ecu_rust_test.sh` | `uds-ecu-rust` 테스트 환경 전체 정리 |

## 기본 퍼징 개발 환경

### `Dockerfile`

Ubuntu 이미지에 다음 개발 도구를 설치합니다.

- `clang`: libFuzzer와 Sanitizer를 사용하는 C 컴파일러
- `build-essential`: 기본 컴파일 및 빌드 도구
- `git`: 저장소와 submodule 관리
- `curl`, `ca-certificates`: HTTPS 다운로드
- `vim`: 컨테이너 내부 편집기

작업 디렉터리는 `/work`입니다.

```dockerfile
WORKDIR /work
CMD ["bash"]
```

### `compose.yml`

`Dockerfile`로 `uds-fuzz` 이미지를 만들고 컨테이너를 계속 실행합니다.

```yaml
command: sleep infinity
volumes:
  - .:/work
```

볼륨 `.:/work`의 의미:

```text
호스트 프로젝트 폴더 <-> 컨테이너 /work
```

호스트에서 수정한 파일이 컨테이너에 바로 보이고, 컨테이너에서 수정한 파일도 호스트에 반영됩니다.

### 자동 실행 및 접속

프로젝트 루트에서 실행합니다.

```bash
./docker_run.sh
```

스크립트가 다음 명령을 수행합니다.

```bash
docker compose up -d
docker compose exec uds-fuzz bash
```

- `up -d`: 컨테이너를 백그라운드로 실행
- `exec`: 실행 중인 컨테이너 안에서 Bash 실행

Bash에서 나오려면 다음을 입력합니다.

```bash
exit
```

컨테이너 종료 및 삭제:

```bash
docker compose down
```

## Stateless ISO-TP 테스트 환경

### `docker/isotp-stateless.yml`

두 서비스를 실행합니다.

```text
vecu
└── Rust isotp_stateless_ecu 실행

isotp-stateless-test
└── can-utils로 UDS 요청 전송 및 응답 검사
```

두 서비스에는 다음 설정이 있습니다.

```yaml
network_mode: host
```

별도 Docker 네트워크 대신 Linux 호스트의 네트워크 namespace를 사용하므로 두 컨테이너가 호스트의 `vcan0`에 접근할 수 있습니다.

### vECU Dockerfile

파일: `ecu-projects/ecu-demo/rust/Dockerfile.isotp`

Rust 소스와 Cargo 파일을 이미지에 복사하고 stateless example을 빌드합니다.

```dockerfile
RUN cargo build --release --example isotp_stateless_ecu
CMD ["cargo", "run", "--release", "--example", "isotp_stateless_ecu"]
```

- `RUN`: Docker 이미지를 빌드할 때 Rust example 컴파일
- `CMD`: 컨테이너를 시작할 때 Rust example 실행

### 테스트 Dockerfile

파일: `tests/integration/isotp_stateless/Dockerfile`

Ubuntu에 `can-utils`를 설치하고 `test_uds.sh`를 복사합니다.

```dockerfile
ENTRYPOINT ["/test_uds.sh"]
```

컨테이너가 시작되면 `test_uds.sh`가 자동으로 실행됩니다.

### 컨테이너 통신 구조

```text
isotp-stateless-test                   vecu

isotpsend
  └── 22 F1 90, CAN ID 0x7E0 ───────> socket.read()
                                         │
                                  handle_request()
                                         │
isotprecv                               │
  └── 62 F1 90 01 02 03 04 <──────── socket.write()
      CAN ID 0x7E8
```

두 컨테이너 사이에서 ISO-TP 데이터를 전달하는 것은 Linux 커널의 SocketCAN과 `vcan0`입니다.

## Stateless ISO-TP 자동 실행

Linux에서 프로젝트 루트 기준으로 실행합니다.

```bash
./scripts/run_isotp_stateless_test.sh
```

스크립트가 자동으로 처리하는 작업:

1. `vcan` 커널 모듈 로드
2. `vcan0` 생성 및 활성화
3. Docker 이미지 빌드
4. `vecu` 실행
5. `isotp-stateless-test` 실행
6. UDS 응답 검사
7. 테스트 컨테이너 정리

성공 결과:

```text
PASS: 62 F1 90 01 02 03 04
```

## Stateless ISO-TP 수동 실행

`vcan0` 준비:

```bash
sudo modprobe vcan
sudo ip link add dev vcan0 type vcan
sudo ip link set dev vcan0 up
```

이미 `vcan0`가 존재하면 `ip link add`는 생략합니다.

이미지 빌드:

```bash
docker compose -f docker/isotp-stateless.yml build
```

vECU 실행:

```bash
docker compose -f docker/isotp-stateless.yml up -d vecu
```

테스트 실행:

```bash
docker compose -f docker/isotp-stateless.yml run --rm isotp-stateless-test
```

vECU 로그 확인:

```bash
docker compose -f docker/isotp-stateless.yml logs vecu
```

컨테이너 정리:

```bash
docker compose -f docker/isotp-stateless.yml down
```

이미지와 `vcan0`까지 전체 정리:

```bash
./scripts/cleanup_isotp_stateless_test.sh
```

## uds-ecu-rust 자동 통신 테스트

실제 `uds-ecu-rust`는 세션과 Seed 상태를 유지하므로 요청 네 개를 같은 ECU 컨테이너에 순서대로 보냅니다.

```text
10 03          -> 50 03
22 F1 90       -> 62 F1 90 01 02 03 04
27 01          -> 67 01 12 34
27 02 B8 9E    -> 67 02
```

자동 실행:

```bash
./scripts/run_uds_ecu_rust_test.sh
```

수동 실행:

```bash
docker compose -f docker/uds-ecu-rust.yml build
docker compose -f docker/uds-ecu-rust.yml up -d uds-ecu-rust
docker compose -f docker/uds-ecu-rust.yml run --rm uds-ecu-rust-test
docker compose -f docker/uds-ecu-rust.yml down
```

전체 정리:

```bash
./scripts/cleanup_uds_ecu_rust_test.sh
```

## 자주 사용하는 Docker 명령

```bash
# 실행 중인 컨테이너 확인
docker ps

# 종료된 컨테이너까지 확인
docker ps -a

# 이미지 확인
docker images

# Compose 서비스 로그 확인
docker compose logs

# Compose 컨테이너 종료 및 삭제
docker compose down
```

## 주의 사항

- SocketCAN과 `vcan`은 Linux 커널 기능입니다.
- Stateless ISO-TP 테스트는 Linux 호스트 또는 Linux VM에서 실행해야 합니다.
- `network_mode: host`의 동작은 Docker Desktop의 macOS 및 Windows 환경과 다릅니다.

ISO-TP 요청과 Rust 라이브러리의 상세 통신 과정은 [Stateless ISO-TP 통합 테스트](isotp_stateless_test.md)를 참고하세요.
