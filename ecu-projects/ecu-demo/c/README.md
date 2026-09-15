# Stateless C ECU MVP

간단한 C 코드로 UDS 요청 처리, libFuzzer 하네스와 NRC semantic oracle의 역할을 학습하는 예제입니다.

현재 범위는 `ReadDataByIdentifier (SID 0x22)` 한 개이며 CAN 또는 ISO-TP 통신은 사용하지 않습니다.

## 구조

```text
c/
├── README.md
├── stateless_ecu_mvp.c   # ECU 요청 처리와 수동 테스트 main
├── fuzz/                  # 메모리 안전성 기본 퍼저
│   ├── Dockerfile
│   ├── fuzz_target.c
│   ├── run.sh
│   └── run_fuzzer.sh
└── fuzz-nrc/              # 응답 및 NRC 정확성 퍼저
    ├── README.md
    ├── Dockerfile
    ├── fuzz_target.c
    ├── nrc_oracle.c
    ├── nrc_oracle.h
    ├── run.sh
    └── run_fuzzer.sh
```

## ECU 처리 흐름

```text
요청 배열과 길이
→ handle_request()
→ Response.data에 응답 저장
→ Response.len에 실제 응답 길이 저장
```

지원하는 응답 규칙:

```text
빈 요청          → 응답 없음
0x22 이외 SID    → 7F [SID] 11
잘못된 0x22 길이 → 7F 22 13
지원하지 않는 DID → 7F 22 31
22 F1 90         → 62 F1 90 01
```

`0x11`, `0x13`, `0x31`은 각각 다음 NRC를 의미합니다.

```text
0x11 ServiceNotSupported
0x13 IncorrectMessageLengthOrInvalidFormat
0x31 RequestOutOfRange
```

## 일반 실행

`c` 폴더에서:

```bash
cc -std=c11 -Wall -Wextra -Wpedantic stateless_ecu_mvp.c
./a.out
```

`stateless_ecu_mvp.c`의 `main()`에 있는 `request` 배열을 바꾸면 다른 요청을 수동으로 확인할 수 있습니다.

## 기본 퍼징

```bash
./fuzz/run.sh
```

Docker 안에서 Clang, libFuzzer, ASan과 UBSan을 사용합니다. 퍼저가 생성한 바이트를 `handle_request()`에 전달하고 다음 항목을 확인합니다.

```text
프로세스 crash
잘못된 메모리 접근
Undefined Behavior
Response.len의 배열 용량 초과
```

## NRC semantic 퍼징

```bash
./fuzz-nrc/run.sh
```

기본 퍼저와 달리 `nrc_oracle.c`가 입력별 기대 응답을 정의합니다.

```text
fuzz 입력
→ fuzz_target.c 하네스
→ handle_request() 테스트 대상
→ nrc_oracle.c 기대값 비교
```

실제 응답이 기대 SID, NRC 또는 길이와 다르면 `abort()`를 호출하고 libFuzzer가 해당 입력을 `artifacts/`에 저장합니다.

## Stateless의 의미

입력 하나는 독립적인 UDS 요청 하나입니다. 이전 입력의 session, security level 또는 transfer 상태를 저장하지 않습니다. 여러 요청의 순서가 필요한 검사는 이후 Stateful 하네스로 분리합니다.

## 생성 파일

다음 폴더는 실행 중 만들어지며 Git에 포함하지 않습니다.

```text
fuzz/{artifacts,build,corpus,logs}/
fuzz-nrc/{artifacts,build,corpus,logs}/
```
