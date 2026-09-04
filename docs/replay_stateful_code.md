# Stateful Replay 코드 이해

Stateful Replay 예제는 이해하기 쉬운 순서로 세 단계로 나뉩니다.

```text
replay_stateful_simple.rs
→ 파일 하나를 Replay하는 핵심 코드

replay_stateful_summary.rs
→ 폴더 전체의 단순 통계 코드

replay_stateful.rs
→ 목표 시퀀스 탐색을 포함한 전체 코드
```

처음에는 `replay_stateful_simple.rs`, 다음에는 `replay_stateful_summary.rs`를 읽는 것을 권장합니다. 이 문서의 자세한 함수 설명은 전체 기능이 있는 `replay_stateful.rs`를 기준으로 합니다.

파일 위치:

```text
ecu-projects/uds-ecu-rust/examples/
```

Replay 사용법과 결과 해석은 [Stateful 퍼징 결과 확인](cargo_stateful_fuzz_results.md)을 참고하세요.

## 프로그램의 목적

`stateful-sequence` 퍼징 하네스는 많은 입력을 빠르게 처리하기 위해 `handle_request()`의 응답을 버립니다.

```rust
let _ = handle_request(&mut state, payload);
```

Replay 프로그램은 저장된 corpus를 같은 규칙으로 다시 실행하고 요청과 응답을 출력합니다.

```rust
let response = handle_request(&mut state, payload);
```

실제 CAN이나 ISO-TP 통신을 하는 것이 아니라 Rust ECU 함수를 직접 호출합니다.

## 전체 호출 흐름

```text
main()
  ↓
run()
  ↓
입력 경로 확인
  ├─ 파일 → replay_file()
  │            ↓
  │          replay()
  │
  └─ 폴더 → replay_directory()
               ↓
             replay_file()
               ↓
             replay()
```

코드는 역할에 따라 네 부분으로 나뉩니다.

```text
핵심 기능
→ next_record(), replay(), execute_record(), hex()

부가 분석 기능
→ record_positive_response(), SequenceTracker

부가 편의 기능
→ replay_file(), replay_directory(), 통계 및 출력 함수

실행 제어
→ run(), parse_show_all(), main()
```

핵심 기능은 corpus를 실제 ECU에 다시 전달하는 데 필요합니다. 부가 기능은 전체 corpus 통계, 목표 시퀀스 탐색과 편리한 명령행 실행을 위해 존재합니다.

각 함수의 역할:

- `next_record()`: `Action + Length + Payload` 레코드 하나를 파싱합니다.
- `replay()`: corpus 파일 하나의 전체 실행을 관리합니다.
- `execute_record()`: Action에 따라 UDS 요청, RESET 또는 무시 처리를 합니다.
- `hex()`: 바이트 배열을 읽기 쉬운 16진수 문자열로 변환합니다.
- `record_positive_response()`: 관심 있는 Positive Response 발견 여부를 기록합니다.
- `SequenceTracker::observe()`: DID와 SecurityAccess 시퀀스 단계를 추적합니다.
- `replay_file()`: corpus 파일 하나를 읽고 Replay합니다.
- `replay_directory()`: 폴더 안의 corpus를 하나씩 실행하고 결과를 합칩니다.
- `add_to_directory_summary()`: 파일 하나의 결과를 폴더 통계에 더합니다.
- `print_*()`: 상세 결과, 폴더 통계와 시퀀스를 출력합니다.
- `run()`: 명령행 인수를 읽고 파일 모드와 폴더 모드를 선택합니다.
- `main()`: 프로그램을 시작하고 최종 오류를 출력합니다.

## 입력 레코드

Stateful corpus는 다음 레코드의 반복입니다.

```text
[Action: 1바이트][Length: 2바이트 Big Endian][Payload]
```

코드에서 Action을 상수로 정의합니다.

```rust
const SEND_REQUEST: u8 = 0x01;
const RESET: u8 = 0x04;
```

상수를 사용하면 코드 곳곳에 의미를 알기 어려운 `0x01`, `0x04`를 직접 쓰지 않아도 됩니다.

## 결과 구조체

### `ReplayResult`

```rust
#[derive(Default)]
struct ReplayResult {
    valid_requests: usize,
    resets: usize,
    unknown_actions: usize,
    trailing_bytes: usize,
    // 생략
}
```

Corpus 파일 하나를 실행한 결과입니다.

- `valid_requests`: 실행한 `SEND_REQUEST` 개수
- `resets`: 실행한 `RESET` 개수
- `unknown_actions`: 무시한 Action 개수
- `trailing_bytes`: 완전한 레코드로 읽지 못한 마지막 바이트 수
- `positive_*`: 특정 Positive Response가 있었는지 표시
- `did_sequence`: DID 성공 시퀀스 발견 여부
- `security_sequence`: SecurityAccess 성공 시퀀스 발견 여부

### `DirectorySummary`

폴더 안의 여러 `ReplayResult`를 합친 결과입니다.

```rust
struct DirectorySummary {
    files_scanned: usize,
    positive_50_03: usize,
    did_sequence_files: Vec<PathBuf>,
    // 생략
}
```

응답을 발견한 파일 수와 목표 시퀀스가 들어 있는 파일 경로를 저장합니다.

## `#[derive(Default)]`

```rust
#[derive(Default)]
```

Rust가 구조체의 기본값 생성 기능을 자동으로 만들어 달라는 뜻입니다.

```rust
let result = ReplayResult::default();
```

현재 필드의 기본값은 다음과 같습니다.

```text
usize → 0
bool  → false
Vec   → 빈 벡터
```

모든 필드를 직접 초기화하지 않아도 되므로 결과를 모으는 구조체에 유용합니다.

## `hex()` 함수

```rust
fn hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}
```

변환 과정:

```text
[0x10, 0x03]
→ 각 바이트를 대문자 2자리 16진수로 변환
→ ["10", "03"]
→ 공백으로 연결
→ "10 03"
```

주요 문법:

- `&[u8]`: 바이트 slice를 빌려 받습니다.
- `.iter()`: 바이트를 하나씩 순회합니다.
- `.map(...)`: 각 바이트를 문자열로 변환합니다.
- `|byte|`: 하나의 바이트를 받는 closure입니다.
- `02X`: 대문자 16진수를 최소 두 자리로 표시합니다.
- `.collect::<Vec<_>>()`: 변환된 문자열을 벡터로 모읍니다.
- `.join(" ")`: 문자열 사이를 공백으로 연결합니다.

## `replay()`의 상태 유지

```rust
let mut state = EcuState::new();
```

`EcuState`는 corpus 파일 하나를 처리하기 전에 한 번 생성합니다. 따라서 같은 파일 안의 요청은 동일한 ECU 상태를 공유합니다.

```text
10 03
→ session이 0x03으로 변경됨
→ 같은 state에 22 F1 90 전달
→ Extended Session 조건을 만족함
```

폴더의 다음 corpus 파일을 처리할 때는 `replay()`가 다시 호출되므로 새로운 ECU 상태에서 시작합니다.

## `usize`란?

`usize`는 배열 위치, 데이터 길이와 개수를 나타내는 음수가 없는 정수 타입입니다.

```rust
let action = data[offset];
```

배열 위치에 사용하는 `offset`과 `data.len()`의 타입이 `usize`입니다.

Corpus Length는 2바이트이므로 `u16`으로 읽은 후, slice 길이에 사용하기 위해 `usize`로 바꿉니다.

```rust
let length = u16::from_be_bytes([high, low]) as usize;
```

## 레코드 헤더 읽기

`next_record()`가 파싱을 전담합니다.

```rust
if data.len() - *offset < 3 {
    return Err(data.len() - record_start);
}
```

Action 1바이트와 Length 2바이트, 총 3바이트가 남아 있는지 먼저 확인합니다.

```rust
let action = data[*offset];
let length = u16::from_be_bytes([
    data[*offset + 1],
    data[*offset + 2],
]) as usize;
```

`u16::from_be_bytes()`는 두 개의 Big Endian 바이트를 `u16` 숫자로 합칩니다.

```text
[00, 03] → 3
[01, 00] → 256
```

`as usize`는 slice 위치와 길이에 사용할 수 있도록 `u16`을 `usize`로 변환합니다.

## 잘린 입력 확인

```rust
if length > data.len() - *offset {
    return Err(data.len() - record_start);
}
```

레코드가 선언한 Payload 길이보다 실제 남은 바이트가 적으면 slice 범위를 벗어날 수 있습니다. 이를 먼저 확인하고 안전하게 Replay를 종료합니다.

```text
01 00 04 27 01
```

위 입력은 Payload 길이를 4로 선언했지만 `27 01` 두 바이트만 남았으므로 잘린 레코드입니다.

## Payload 분리

```rust
let payload = &data[*offset..*offset + length];
*offset += length;
```

전체 corpus에서 현재 레코드의 UDS Payload 부분만 빌립니다. 데이터를 복사하지 않고 원본의 일부를 참조합니다.

## Action 실행

`execute_record()`가 Action별 실행을 전담합니다.

```rust
match record.action {
    SEND_REQUEST => {
        let response = handle_request(state, record.payload);
    }
    RESET => {
        *state = EcuState::new();
    }
    _ => {
        result.unknown_actions += 1;
    }
}
```

- `SEND_REQUEST`: Payload를 ECU에 전달합니다.
- `RESET`: ECU 상태와 시퀀스 추적 단계를 초기화합니다.
- `_`: 정의하지 않은 모든 Action을 의미하며 실행하지 않고 개수만 기록합니다.

## `starts_with()` 응답 확인

```rust
response.starts_with(&[0x62, 0xF1, 0x90])
```

응답의 시작 바이트가 지정한 값과 같은지 확인합니다.

DID 응답 뒤의 데이터가 달라져도 다음 공통 부분으로 응답 종류를 식별할 수 있습니다.

```text
62 F1 90 01 02 03 04
^^^^^^^^
확인하는 부분
```

## `|=` 연산자

```rust
result.positive_50_03 |= response.starts_with(&[0x50, 0x03]);
```

Boolean 값에서 `|=`는 이전 값 또는 새 조건 중 하나라도 `true`이면 결과를 `true`로 유지합니다.

```text
이전에 발견하지 않음 + 이번에도 아님 → false
이전에 발견하지 않음 + 이번에 발견 → true
이전에 이미 발견함 + 이후 응답이 다름 → true 유지
```

즉, 파일 안에서 해당 응답이 한 번이라도 나왔는지를 기록합니다.

## 시퀀스 단계 추적

```rust
let mut did_stage = 0;
let mut security_stage = 0;
```

`did_stage`는 다음 순서를 확인합니다.

```text
0
→ 50 03 응답 발견
→ 1
→ 62 F1 90 응답 발견
→ DID 시퀀스 성공
```

`security_stage`는 다음 순서를 확인합니다.

```text
0
→ 50 03 발견
→ 1
→ 67 01 발견
→ 2
→ 67 02 발견
→ SecurityAccess 시퀀스 성공
```

`RESET`이나 `50 01` 응답으로 Default Session에 진입하면 추적 단계를 `0`으로 초기화합니다.

## `Path`와 `PathBuf`

```rust
fn replay_file(path: &Path, detailed: bool)
```

`Path`는 함수가 전달받은 경로를 빌려서 사용할 때 적합합니다.

```rust
did_sequence_files: Vec<PathBuf>
```

`PathBuf`는 구조체가 파일 경로를 직접 소유하고 저장해야 할 때 사용합니다.

간단히 구분하면:

```text
&Path   → 경로를 잠시 빌려 사용
PathBuf → 경로를 소유하고 저장
```

## `Result`와 `?`

```rust
fn replay_file(path: &Path, detailed: bool) -> io::Result<ReplayResult> {
    let data = fs::read(path)?;
    // 생략
    Ok(result)
}
```

`io::Result<ReplayResult>`는 다음 두 결과 중 하나를 반환한다는 뜻입니다.

```text
Ok(ReplayResult) → 파일을 읽고 정상 처리
Err(io::Error)   → 파일 읽기 실패
```

`?`는 오류가 발생하면 현재 함수를 즉시 종료하고 그 오류를 호출한 함수로 전달합니다.

## 폴더 파일 수집

```rust
let mut files = fs::read_dir(directory)?
    .filter_map(|entry| entry.ok())
    .map(|entry| entry.path())
    .filter(|path| path.is_file())
    .collect::<Vec<_>>();
```

처리 순서:

```text
폴더 항목 읽기
→ 읽기에 성공한 항목만 선택
→ 항목을 파일 경로로 변환
→ 일반 파일만 선택
→ Vec<PathBuf>로 수집
```

```rust
files.sort();
```

항상 같은 순서로 출력되도록 경로를 정렬합니다.

## `usize::from(bool)`

```rust
summary.positive_50_03 += usize::from(result.positive_50_03);
```

`bool`을 숫자로 바꿉니다.

```text
false → 0
true  → 1
```

따라서 응답이 발견된 파일마다 `1`을 더하여 응답 횟수가 아닌 **파일 수**를 계산합니다.

## 명령행 인수 처리

```rust
let args = env::args().collect::<Vec<_>>();
```

프로그램에 전달한 명령행 인수를 문자열 벡터로 모읍니다.

예:

```bash
cargo run --example replay_stateful -- fuzz/corpus/stateful-sequence --all
```

Replay 프로그램이 받는 값:

```text
args[0] → 실행 프로그램 경로
args[1] → fuzz/corpus/stateful-sequence
args[2] → --all
```

Cargo 명령의 `--` 뒤에 있는 값만 Replay 프로그램의 인수로 전달됩니다.

## 파일과 폴더 선택

```rust
if path.is_file() {
    replay_file(path, true)?;
} else if path.is_dir() {
    replay_directory(path, show_all)?;
}
```

- 파일이면 해당 corpus를 상세 출력합니다.
- 폴더이면 기본적으로 전체 요약을 출력합니다.
- 폴더와 `--all`을 함께 전달하면 모든 파일을 상세 출력합니다.

## `main()`과 종료 코드

```rust
fn main() {
    if let Err(error) = run() {
        eprintln!("Error: {error}");
        process::exit(1);
    }
}
```

- `Ok(())`: 정상적으로 종료합니다.
- `Err(error)`: 오류를 표준 오류 출력에 표시합니다.
- `process::exit(1)`: 셸에 프로그램이 실패했음을 알립니다.

종료 코드 `0`은 성공, `0`이 아닌 값은 실패를 의미합니다.

## 실행 방법

`uds-ecu-rust` 폴더에서 실행합니다.

핵심 코드로 파일 하나 확인:

```bash
cargo run --example replay_stateful_simple -- \
  fuzz/corpus/stateful-sequence/<corpus-file>
```

간단한 폴더 전체 요약:

```bash
cargo run --example replay_stateful_summary -- \
  fuzz/corpus/stateful-sequence
```

전체 기능으로 파일 하나 상세 확인:

```bash
cd ecu-projects/uds-ecu-rust
```

파일 하나 상세 확인:

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence/<corpus-file>
```

폴더 전체 요약:

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence
```

폴더 전체 상세 출력:

```bash
cargo run --example replay_stateful -- \
  fuzz/corpus/stateful-sequence --all
```

## 핵심 정리

```text
stateful-sequence.rs
→ 많은 입력을 빠르게 실행하여 corpus 생성

replay_stateful.rs
→ corpus를 같은 규칙으로 다시 파싱
→ 같은 EcuState에 요청을 순서대로 전달
→ 요청, 응답, 정상 시퀀스를 출력
```

Replay가 퍼징 당시와 같은 결과를 내려면 두 프로그램의 `Action + Length + Payload` 파싱 규칙이 같아야 합니다.
