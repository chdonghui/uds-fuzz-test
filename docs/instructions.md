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

### iso14229 버전 업데이트

위 초기화 명령은 **프로젝트에 기록된 커밋**을 가져오며, 자동으로 최신 버전을 선택하지 않습니다. 아래 명령은 모두 프로젝트 루트에서 실행합니다.

먼저 서브모듈 내부에 수정한 파일이 없는지 확인합니다. 변경이 있다면 먼저 별도 커밋 등으로 보존하세요.

```bash
git -C third_party/iso14229 status --short
```

#### 방법 1: 최신 릴리스 선택 — 권장

원격 커밋과 태그를 가져온 뒤 버전 목록을 확인합니다.

```bash
git -C third_party/iso14229 fetch origin --tags
git -C third_party/iso14229 tag --sort=-version:refname
```

태그 목록과 [공식 릴리스](https://github.com/driftregion/iso14229/releases)를 확인하고 원하는 최신 정식 릴리스 태그를 선택합니다. 목록 맨 위가 반드시 최신 안정 릴리스인 것은 아닙니다.

`gh`없이 `curl`과 Python 3로 GitHub가 지정한 최신 정식 릴리스 태그만 확인할 수 있습니다.

```bash
curl -fsSL https://api.github.com/repos/driftregion/iso14229/releases/latest \
  | python3 -c 'import json, sys; print(json.load(sys.stdin)["tag_name"])'
```

이 명령은 조회만 하며 서브모듈을 변경하지 않습니다. API 오류나 호출 제한으로 조회가 실패하면 공식 릴리스 페이지에서 확인하세요. 정식 릴리스라는 표시가 버그 없음을 보장하지는 않습니다.

출력된 태그로 전환합니다.

```bash
git -C third_party/iso14229 checkout --detach <릴리스-태그>
```

`<릴리스-태그>`는 실제 확인한 태그로 바꿉니다.

#### 방법 2: 개발 브랜치 최신 커밋 선택

정식 릴리스 이후 개발 중인 변경까지 가져오려면 다음 명령을 사용합니다.

```bash
git submodule update --init --remote third_party/iso14229
```

이 명령은 서브모듈에 설정된 추적 브랜치를 사용하며, 별도 설정이 없다면 원격 기본 브랜치를 사용합니다. 최신 릴리스 태그를 선택하는 명령은 아닙니다.

#### 변경 확인 및 기록

두 방법 중 하나를 실행한 후 변경된 커밋을 확인합니다.

```bash
git submodule status
git diff --submodule=log -- third_party/iso14229
```

API 변경 여부와 빌드·기본 테스트 호환성을 확인한 뒤 상위 저장소에 새 커밋 ID를 기록합니다.

```bash
git add third_party/iso14229
git commit -m "chore: update iso14229 submodule"
```

상위 저장소는 새로 선택한 커밋 하나에 다시 고정됩니다. 이후 원격에 새 버전이 나와도 자동으로 업데이트되지 않습니다.

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
