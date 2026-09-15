# 퍼징 진행 상태와 계획

이 문서는 현재 완료된 퍼징 환경과 앞으로 진행할 순서를 정리합니다.

## 관리 원칙

```text
fuzz-projects/
→ C/C++ 기반 학습·OSS 퍼징 하네스

ecu-projects/uds-ecu-rust/fuzz/
→ Rust ECU의 cargo-fuzz target과 corpus

third_party/
→ 수정하지 않는 upstream OSS 원본
```

퍼징 하네스와 원본 코드를 분리하고, 발견 결과는 정상 입력과 비교한 뒤 재실행 가능한 형태로 기록합니다.

## 완료된 단계

### 1. libFuzzer 기본 동작 확인

위치:

```text
fuzz-projects/libfuzzer-basic/
```

실행:

```bash
./fuzz-projects/libfuzzer-basic/run.sh
```

확인한 내용:

```text
Docker에서 Clang과 libFuzzer 실행
커버리지 기반으로 UDS 문자열 발견
의도적인 SIGABRT 입력을 artifact로 저장
55 44 53 입력 확인
```

### 2. iso14229 Stateless 서버 퍼징

위치:

```text
fuzz-projects/iso14229-server/
```

실행:

```bash
./fuzz-projects/iso14229-server/run.sh
```

입력 규칙:

```text
입력 파일 1개 = raw UDS payload 1개
별도 길이 헤더 없음
입력마다 서버 상태 초기화
```

첫 30초 실행 결과:

```text
약 939만 회 실행
최종 corpus 180개
ASan/UBSan finding 없음
```

오류가 없었다는 것은 해당 실행에서 sanitizer가 문제를 찾지 못했다는 뜻이며, 라이브러리 전체의 안전성을 증명하지는 않습니다.


## 다음 작업

### 1. 가변 길이 UDS 요청의 Boundary 검사 확장

현재 고정 인덱스 메시지 경계 oracle을 가변 길이 필드가 있는 서비스로 확장합니다.

확인할 조건:

```text
선언 길이와 실제 남은 길이가 일치하는가?
동적 인덱스가 실제 메시지 범위 안에 있는가?
콜백의 pointer + length가 수신 메시지 범위 안에 있는가?
요청 밖 poison 값이 파싱 결과에 영향을 주는가?
```

이 저장소에서는 재사용 가능한 Boundary oracle을 `fuzz-projects/` 아래의 독립 퍼저로 구현합니다.

### 2. Boundary oracle 일반화

서비스마다 전체 구조체를 비교하지 않고 의미 있는 값만 정규화합니다.

```text
이벤트 종류
DID 또는 DTC
파싱된 길이
콜백에 전달된 offset과 size
생성된 응답 바이트
```

포인터 주소와 구조체 padding은 실행마다 달라질 수 있으므로 직접 비교하지 않습니다.

### 3. Stateful iso14229 서버 퍼저

Stateless 검사가 안정된 뒤 여러 UDS 요청을 하나의 서버 상태에 순서대로 전달합니다.

대상 흐름:

```text
DiagnosticSessionControl
→ SecurityAccess
→ RequestDownload
→ TransferData
→ RequestTransferExit
```

계획 중인 입력 형식:

```text
[u16 Big Endian 요청 길이][raw UDS 요청]
[u16 Big Endian 요청 길이][raw UDS 요청]
...
```

길이 prefix는 요청 경계를 명확하게 유지하며 다른 OSS 하네스에도 적용하기 쉽습니다. 구현 전 최대 요청 수와 전체 입력 크기를 제한합니다.

### 4. Coverage 확인

단순 `cov` 숫자뿐 아니라 실제로 어떤 함수와 분기가 실행됐는지 확인하는 보고서를 추가합니다.

```text
서비스별 도달 여부
이벤트 콜백 도달 여부
정상 seed가 추가한 경로
장시간 실행에서 더 이상 증가하지 않는 경로
```

Coverage는 안전성을 증명하는 값이 아니라 퍼징하지 못한 영역을 찾는 용도로 사용합니다.

### 5. Upstream 제보 준비

재현된 후보마다 다음 자료를 준비합니다.

```text
영향받는 정확한 commit
최소 입력
정상 대조군
ASan 또는 semantic oracle 결과
공개 API를 통한 도달 경로
발생 조건과 영향 범위
권장 경계 검사
```

미수정 문제는 먼저 upstream에 비공개로 전달하고 수정 일정이 정해진 뒤 공개 여부를 결정합니다.

### 6. 두 번째 OSS 대상으로 확장

`iso14229` 하네스와 결과 해석 절차가 안정된 뒤 다른 UDS 구현을 추가합니다.

우선 비교할 내용:

```text
입력 API 연결 방법
Stateless와 Stateful 상태 관리
NRC 처리
가변 길이 필드 검사
corpus 재사용 가능 여부
sanitizer 적용 가능 여부
```

특정 도구 이름보다 소스 빌드와 함수 단위 퍼징이 가능한 OSS를 우선합니다.

### 7. QEMU와 실제 펌웨어 분석

QEMU는 C 라이브러리 함수 퍼징보다 비용이 크므로 마지막 단계로 둡니다.

```text
먼저: Docker + 함수 단위 libFuzzer
이후: 다른 아키텍처 또는 펌웨어 실행이 필요할 때 QEMU
```

## 현재 우선순위

```text
1. 가변 길이 Boundary 퍼저
2. Boundary oracle 일반화
3. Stateful 서버 퍼저
4. Coverage 보고서
5. Upstream 제보 준비
6. 두 번째 OSS
7. QEMU
```

현재 바로 진행할 작업은 **가변 길이 UDS 요청에 대한 Boundary 검사 확장**입니다.
