# Stateful 퍼징 결과 확인

`stateful-sequence`가 생성한 corpus를 해석하고, 각 UDS 요청의 실제 응답을 확인하는 방법입니다.

Replay 예제는 난이도와 기능에 따라 세 개로 나뉩니다.

```text
examples/replay_stateful_simple.rs
→ corpus 파일 하나의 요청과 응답 출력

examples/replay_stateful_summary.rs
→ corpus 폴더의 파일, 요청, 응답 개수 요약

examples/replay_stateful.rs
→ 폴더 통계와 목표 UDS 시퀀스까지 분석
```

아래 명령은 `ecu-projects/uds-ecu-rust`에서 실행합니다.

## 확인할 결과

Stateful 퍼징 결과는 다음 세 가지로 나누어 확인합니다.

```text
퍼징 로그
→ 실행 횟수, coverage 변화, 크래시 여부 확인

corpus
→ 퍼저가 새로운 실행 특징을 발견한 입력 시퀀스

artifacts
→ crash, timeout, OOM 등을 일으킨 입력
```

Corpus에 정상처럼 보이는 요청이 있어도 정상 응답 분기까지 실행됐다고 바로 단정할 수는 없습니다. 요청을 같은 ECU 상태에 다시 전달하고 응답을 확인해야 합니다.

## Corpus 입력 형식

현재 `stateful-sequence` 입력은 다음 레코드의 반복입니다.

```text
[Action: 1바이트][Length: 2바이트 Big Endian][Payload]
```

현재 사용하는 Action은 다음과 같습니다.

```text
01    SEND_REQUEST
04    RESET
```

예:

```text
01 00 02 10 03 01 00 03 22 F1 90
```

레코드별로 해석하면:

```text
01 00 02 10 03
→ SEND_REQUEST, 길이 2, UDS 요청 10 03

01 00 03 22 F1 90
→ SEND_REQUEST, 길이 3, UDS 요청 22 F1 90
```

`Action`과 `Length`는 하네스 전용 정보입니다. ECU의 `handle_request()`에는 `10 03`, `22 F1 90` 같은 UDS Payload만 전달됩니다.

## Replay 프로그램의 원리

Replay 프로그램은 cargo-fuzz가 저장한 corpus 파일을 다시 읽어 퍼징 당시 동작을 재현하는 분석 도구입니다.

```text
corpus 파일 읽기
→ Action, Length, Payload 파싱
→ EcuState를 한 번 생성
→ Payload를 순서대로 handle_request()에 전달
→ 요청과 반환된 응답 출력
```

퍼징 하네스는 빠른 실행을 위해 응답을 버립니다.

```rust
let _ = handle_request(&mut state, payload);
```

Replay 프로그램은 응답을 저장하고 출력합니다.

```rust
let response = handle_request(&mut state, payload);

println!("Request:  {}", hex(payload));
println!("Response: {}", hex(&response));
```

`hex()`는 바이트 배열을 `10 03`, `62 F1 90`처럼 공백으로 구분된 16진수 문자열로 바꿉니다.

Replay는 실제 CAN이나 ISO-TP 통신을 수행하지 않습니다. Corpus의 UDS Payload를 동일한 `handle_request()` 함수에 다시 전달합니다.

## 퍼징 하네스와 Replay의 역할

`fuzz/fuzz_targets/stateful-sequence.rs`의 역할:

```text
수많은 입력을 빠르게 실행
새로운 coverage 탐색
크래시 탐색
corpus와 artifact 생성
```

`examples/replay_stateful.rs`의 역할:

```text
선택한 corpus를 재실행
레코드를 사람이 읽을 수 있게 해석
각 UDS 요청과 응답 출력
정상 상태 전이 여부 확인
```

퍼징 하네스 안에서 `println!()`으로 모든 응답을 출력하면 퍼징 속도가 크게 느려집니다. 따라서 하네스는 그대로 유지하고 Replay를 별도로 두는 것이 좋습니다.

## Replay 실행 방식

### 학습용 파일 하나 확인

```bash
cargo run --example replay_stateful_simple -- \
  fuzz/corpus/stateful-sequence/<corpus-file>
```

Replay의 핵심인 레코드 파싱, 상태 유지, 요청·응답 출력을 가장 단순한 코드로 확인합니다.

### 학습용 폴더 요약

```bash
cargo run --example replay_stateful_summary -- \
  fuzz/corpus/stateful-sequence
```

다음 항목만 출력합니다.

```text
폴더 안의 파일 개수
전체 SEND_REQUEST 개수
Positive Response 개수
Negative Response 개수
No Response 개수
```

### 전체 기능: 파일 하나 상세 확인

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence/<corpus-file>
```

선택한 파일의 모든 유효한 요청과 응답을 순서대로 출력합니다.

예상 출력:

```text
File: dfd295cb7ef060dade44018dc06dbf58fbb0694f

[0] SEND_REQUEST
    Request:  10 03
    Response: 50 03

[1] SEND_REQUEST
    Request:  22 F1 90
    Response: 62 F1 90 01 02 03 04
```

특정 정상 시퀀스나 artifact를 자세히 확인할 때 사용합니다.

### 폴더 전체 요약

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence
```

폴더의 모든 파일을 상세 출력하면 결과가 너무 길어질 수 있으므로, 기본 폴더 모드는 요약만 출력하는 것이 좋습니다.

현재 corpus를 실행한 결과:

```text
Files scanned: 235
Files containing valid actions: 226

Positive responses (number of files):
  50 03: 48
  62 F1 90: 6
  67 01: 14
  67 02: 0

10 03 -> 22 F1 90: found in 6 file(s)
10 03 -> 27 01 -> 27 02 B8 9E: not found
```

이 숫자는 응답 횟수가 아니라 해당 Positive Response가 한 번 이상 나온 **파일 수**입니다.

발견한 시퀀스의 대표 파일 이름도 함께 표시하면 파일 하나 상세 모드로 다시 확인할 수 있습니다.

### 폴더 전체 상세 확인

필요한 경우에만 `--all` 옵션으로 모든 파일의 상세 내용을 출력할 수 있습니다.

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence --all
```

Corpus가 많거나 파일이 길면 출력량이 매우 커지므로 기본 사용 방식으로는 권장하지 않습니다.

## 정상 시퀀스 판단 기준

정상 요청을 발견했다고 판단하려면 다음 조건을 모두 확인해야 합니다.

1. 목표 요청들이 corpus에 올바른 순서로 존재해야 합니다.
2. 요청들이 하나의 동일한 `EcuState`에 전달되어야 합니다.
3. 각 요청에서 기대한 Positive Response가 반환되어야 합니다.
4. 중간에 `RESET`이 있어 필요한 상태가 초기화되지 않았는지 확인해야 합니다.

Extended Session 진입 후 DID 읽기 성공 예:

```text
10 03
→ 50 03

22 F1 90
→ 62 F1 90 01 02 03 04
```

SecurityAccess 성공 예:

```text
10 03
→ 50 03

27 01
→ 67 01 12 34

27 02 B8 9E
→ 67 02
```

Corpus에 `27 02 B8 9E`가 들어 있어도 앞에서 필요한 세션 진입과 Seed 요청이 없거나 응답이 Negative Response라면 전체 SecurityAccess 성공 시퀀스를 발견한 것은 아닙니다.

## 응답 해석

일반적인 UDS Positive Response SID는 요청 SID에 `0x40`을 더한 값입니다.

```text
10 → 50
22 → 62
27 → 67
```

Negative Response는 다음 형식입니다.

```text
7F [요청 SID] [NRC]
```

예:

```text
7F 22 31
```

해석:

```text
7F    Negative Response
22    실패한 요청 SID
31    NRC
```

정확한 NRC 의미는 ECU 구현과 UDS 규칙을 함께 확인해야 합니다.

## 의미 없는 레코드 처리

Corpus에는 알 수 없는 Action이나 긴 Payload가 포함될 수 있습니다. 이것은 libFuzzer가 UDS 의미가 아니라 coverage와 실행 특징을 기준으로 corpus를 보존하기 때문입니다.

Replay 출력에서는 다음처럼 처리하는 것이 좋습니다.

```text
SEND_REQUEST와 RESET
→ 상세 출력

알 수 없는 Action
→ 상세 내용은 생략하고 개수만 출력

길이가 맞지 않는 마지막 레코드
→ trailing 또는 truncated bytes로 표시
```

예:

```text
Valid requests: 2
RESET actions: 1
Ignored unknown actions: 7
Trailing bytes: 2
```

Corpus 원본은 수정하지 않습니다. Replay가 유효한 레코드만 골라 보여주는 방식으로 사용합니다.

## Artifact 재실행

Artifact도 Stateful corpus와 동일한 입력 형식이므로 Replay로 해석할 수 있습니다.

```bash
cargo run --example replay_stateful -- \
  fuzz/artifacts/stateful-sequence/crash-<hash>
```

Replay는 크래시 이전의 요청 순서를 이해하는 데 사용합니다. 크래시 자체는 sanitizer가 활성화된 cargo-fuzz 환경에서도 다시 재현해야 합니다.

## 핵심 사용 흐름

```text
cargo +nightly fuzz run stateful-sequence
    ↓
fuzz/corpus/stateful-sequence 확인
    ↓
폴더 Replay로 정상 응답과 목표 시퀀스 요약
    ↓
대표 corpus 파일 하나를 상세 Replay
    ↓
요청 순서, 응답, RESET 여부 확인
```

정리하면:

```text
퍼징 하네스
→ 입력을 찾는 도구

Replay 프로그램
→ 찾은 입력의 의미와 응답을 확인하는 도구
```
