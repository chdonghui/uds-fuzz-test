# iso14229 Stateless Server Fuzzer

공개 OSS인 `iso14229` 서버에 raw UDS 요청 하나를 전달하는 stateless libFuzzer 프로젝트입니다.

## 입력 규칙

```text
fuzz 입력 1개 = UDS payload 1개
별도 길이 헤더 없음
ISO-TP 프레임 없음
입력 최대 길이 = UDS_TP_MTU
```

예를 들어 corpus 파일이 `22 F1 90`이면 그대로 ReadDataByIdentifier 요청으로 전달됩니다.

## 처리 흐름

```text
libFuzzer 입력
→ FuzzTp 수신 콜백
→ UDSServerPoll()
→ UDS 요청 파싱
→ server_event()의 고정된 ECU 응답 데이터
→ FuzzTp 송신 콜백에 응답 저장
```

매 입력마다 `UDSServer_t`를 새로 초기화하므로 이전 입력의 세션과 보안 상태는 이어지지 않습니다.

## 주요 구성

```text
iso14229-server/
├── README.md
├── Dockerfile
├── server_fuzz.c
├── run.sh
├── run_fuzzer.sh
├── replay.sh
├── corpus/       # 실행 중 생성, Git 제외
├── artifacts/    # 오류 입력, Git 제외
├── build/        # 실행 파일, Git 제외
└── logs/         # 실행 로그, Git 제외
```

`run_fuzzer.sh`가 다음 정상 seed를 자동으로 만듭니다.

```text
10 01       DiagnosticSessionControl: default
10 03       DiagnosticSessionControl: extended
22 F1 90    ReadDataByIdentifier
19 02 FF    ReadDTCInformation: report by status mask
```

## 실행

저장소 루트에서:

```bash
./fuzz-projects/iso14229-server/run.sh
```

스크립트가 Docker 이미지 빌드, Clang 컴파일, ASan/UBSan 활성화와 30초 퍼징을 자동으로 수행합니다.

## 결과

오류를 찾지 못하고 제한 시간이 끝난 경우:

```text
PASS: stateless server fuzzing completed without a sanitizer finding.
```

오류를 찾은 경우:

```text
FINDING: libFuzzer saved a failing input.
```

문제 입력은 다음 위치에 저장됩니다.

```text
fuzz-projects/iso14229-server/artifacts/
```

바이트 확인:

```bash
xxd -g 1 fuzz-projects/iso14229-server/artifacts/<artifact-name>
```

재실행:

```bash
./fuzz-projects/iso14229-server/replay.sh <artifact-name>
```

## 확인된 실행 결과

실행일: 2026-09-08
대상: `iso14229` 최신 `main` 커밋 `3526ef3186f254348348e0a546734238766ea8d3`

```text
실행 시간: 31초
실행 횟수: 9,390,878
코드 커버리지 지표: cov 1074
실행 특징 지표: ft 1772
최종 corpus: 180개, 총 3,636바이트
Sanitizer finding: 없음
```

```text
PASS: stateless server fuzzing completed without a sanitizer finding.
```

빌드 중 출력되는 variadic macro와 `tp.c` 마지막 newline 경고는 upstream 원본 경고입니다. 하네스의 Transport 함수 포인터 불일치 경고는 최신 API 적용 후 발생하지 않았습니다.

이번 실행에서 오류를 찾지 못했다는 것은 약 939만 개 입력에서 sanitizer 오류가 없었다는 뜻이며, 라이브러리 전체가 안전하다는 증명은 아닙니다.

## 현재 범위

이 퍼저는 ASan, UBSan, assertion과 프로세스 크래시를 찾는 공개 가능한 일반 하네스입니다.

고정된 서버 내부 배열에서 실제 메시지 길이만 벗어나는 stale read는 ASan이 탐지하지 못할 수 있습니다. 해당 검사는 공개 전 검토가 필요한 별도 boundary 퍼저로 분리합니다.
