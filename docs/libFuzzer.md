# libFuzzer 사용법

## libFuzzer란?

LLVM 프로젝트에 포함된 인프로세스(in-process), 커버리지 가이드(coverage-guided) 방식의 퍼징(Fuzzing) 프레임워크

특징:
- 프로그램에 다양한 입력을 자동으로 생성·변형해서 넣음
- 실행 중 코드 커버리지를 측정함
- 새로운 코드 경로를 발견한 입력을 우선적으로 보존하고 다시 변형함
- 크래시, 비정상 종료, sanitizer 오류를 유발하는 입력을 찾아냄
- 대상 프로그램과 같은 프로세스 안에서 실행되는 in-process fuzzer
- Clang과 통합되어 있어 ASan, UBSan, SanitizerCoverage와 함께 사용하기 편함
- C/C++ 라이브러리나 파서처럼 함수 단위로 직접 입력을 전달할 수 있는 프로그램 퍼징에 특히 적합함

fuzz harness란?

- 퍼저와 실제 테스트 대상 프로그램 사이를 연결해주는 작은 코드
- data를 받아서 실제 테스트하고 싶은 함수에 전달하는 역할을 함.


## 프로젝트 도구 설치

1. 컨테이너 접속

```bash
docker compose run uds-fuzz bash
```

2. 프로젝트 연결 확인

```bash
ls /work/third_party/iso14229/

AUTHORS.txt  LICENSE       docs        iso14229.h                src
BUILD        MODULE.bazel  examples    platforms                 test
CHANGELOG    Makefile      fuzz        scripts                   toolchain
Doxyfile     README.md     iso14229.c  sonar-project.properties  tools
```

3. 패키지 목록 갱신

```bash
apt update
```

4. 최소 개발 도구 설치

```bash
apt install -y clang build-essential git curl ca-certificates
```

- clang
  - libFuzzer / ASan을 사용할 LLVM 컴파일러
  - C/C++ 컴파일러. libFuzzer, ASan, UBSan을 쓰기 위해 필요.
- build-essential
  - gcc, make 등 기본 빌드 도구
  - gcc, g++, make 같은 빌드 도구들 함께 설치
- git
  - 소스 관리
  - iso14229 같은 OSS 저장소를 clone/update하거나 submodule 관리할 때 사용
- curl
  - 필요한 도구 다운로드
  - 인터넷에서 파일이나 설치 스크립트를 내려받을 때 쓰는 명령줄 도구
- ca-certificates
  - HTTPS 인증서 처리
  - HTTPS 사이트의 인증서를 검증하는 데 필요한 인증서 묶음
  - `git clone https://..., curl https://...` 같은 작업이 안전하게 실행

## libFuzzer 동작 확인

가장 간단한 방법은 저장소 루트에서 자동 실행 스크립트를 사용하는 것입니다.

```bash
./fuzz-projects/libfuzzer-basic/run.sh
```

Docker 빌드, Clang 컴파일, 최대 10초 퍼징과 artifact 저장을 자동으로 수행합니다. 발견 결과는 `fuzz-projects/libfuzzer-basic/artifacts/crash`에 저장되며 정상 결과는 `55 44 53`, 즉 `UDS`입니다.

아래는 컨테이너 안에서 직접 빌드하는 수동 방법입니다.

1. 예제 폴더로 이동

```bash
cd /work/fuzz-projects/libfuzzer-basic
```

2. `libfuzzer_basic.c` 테스트 코드 확인

```bash
apt update
apt install -y vim
```

```c
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)
{
    if (size >= 3 &&
        data[0] == 'U' &&
        data[1] == 'D' &&
        data[2] == 'S')
    {
        abort();
    }

    return 0;
}
```

Include Header:
- size_t와 uint8_t를 사용하기 위해 필요한 헤더

`int LLVMFuzzerTestOneInput(const uint8_t *data, size_t size)`:
- libFuzzer가 자동으로 찾아서 반복 호출하는 함수
  - data → libFuzzer가 만든 입력 데이터
  - size → 입력 데이터의 크기
  - uint8_t → 0~255 값을 저장하는 1바이트 정수

`if (size >= 3 &&`
- 입력이 최소 3 이상인지 확인

`data[0] == 'U' && data[1] == 'D' && data[2] == 'S')`
- 입력의 첫 세 바이트가 각각 `U, D, S`인지 검사

`abort();`
- 표준 C 함수로 프로그램에 `SIGABRT`를 발생시켜 의도적으로 종료함
- libFuzzer가 crash 입력을 저장하는지 확인하기 위해 사용


3. 빌드

```bash
clang -g -fsanitize=fuzzer,address libfuzzer_basic.c -o libfuzzer-basic
```

- `clang`
  - C 컴파일러를 실행.
- `-g`
  - 디버깅 정보를 바이너리에 포함.
  - 나중에 crash가 발생했을 때 함수명이나 소스 코드 줄 번호를 보기 쉽게 해줌
- `-fsanitize`
  - 오류 감지하는 검사 기능 켜는 옵션
  - 프로그램에 검사 장치를 붙여서 잘못된 동작을 잡아낼 수 있음
- `-fsanitize=fuzzer,address`
  - 두 기능을 붙여서 빌드.
  - fuzzer는 libFuzzer를 연결
  - address는 ASan(AddressSanitizer)을 활성화해 메모리 오류를 탐지
- `libfuzzer_basic.c`
  - 컴파일할 C 소스 파일.
- `-o libfuzzer-basic`
  - 만들어질 실행 파일의 이름을 `libfuzzer-basic`으로 지정.

4. 실행

```bash
./libfuzzer-basic
```

## iso14229 서버 퍼저 실행

저장소 루트에서 다음 스크립트를 실행합니다.

```bash
./fuzz-projects/iso14229-server/run.sh
```

입력 파일 하나를 raw UDS payload 하나로 사용합니다. Docker에서 Clang, libFuzzer, ASan과 UBSan을 적용해 30초 동안 실행합니다.

구조와 결과 확인 방법은 `fuzz-projects/iso14229-server/README.md`를 참고하세요.

첫 30초 실행에서는 약 939만 회를 수행하고 corpus가 180개로 확장됐으며 ASan/UBSan 오류는 발견되지 않았습니다. 오류 미발견은 전체 안전성을 증명하지 않습니다.

## 이 저장소의 퍼저 구분

```text
fuzz-projects/
→ C/C++ libFuzzer 하네스

ecu-projects/uds-ecu-rust/fuzz/
→ Rust ECU cargo-fuzz target
```

일반 ASan 퍼저는 큰 고정 배열 내부에서 실제 메시지 길이만 벗어나는 stale read를 탐지하지 못할 수 있습니다. 이러한 경우 동일 입력을 서로 다른 tail poison에서 실행하고 정규화된 콜백 결과를 비교하는 semantic oracle이 필요합니다. 해당 검사가 필요하면 `fuzz-projects/` 아래에 별도 Boundary 하네스를 추가합니다.

현재 진행 상태와 이후 순서는 [퍼징 진행 상태와 계획](future.md)을 참고하세요.
