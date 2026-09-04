# uds-ecu-rust에서 cargo-fuzz 사용하기

`cargo-fuzz`로 `ecu-projects/uds-ecu-rust/src/ecu.rs`의 UDS 요청 처리 함수를 검사하는 방법을 설명합니다.

> 현재 저장소에는 `lib.rs`, `fuzz/`, 단일 요청 타깃 `fuzz_target_1`과 Stateful 타깃 `stateful-sequence`가 준비되어 있습니다.

## cargo-fuzz의 역할

`cargo-fuzz`는 Rust 프로젝트에서 libFuzzer를 쉽게 빌드하고 실행하도록 도와주는 도구입니다.

```text
cargo-fuzz
    │ 여러 바이트 입력 생성
    ▼
퍼징 하네스
    │ 입력 전달
    ▼
handle_request()
    │
    └── panic, 비정상 종료, 메모리 오류 탐색
```

`cargo-fuzz`도 내부적으로 libFuzzer를 사용합니다. 따라서 C 퍼징과 마찬가지로 입력을 테스트 대상 함수에 전달하는 **하네스 코드**가 필요합니다.

실행 후 출력과 corpus를 확인하는 방법은 [cargo-fuzz 결과 해석](cargo_fuzz_results.md)을 참고하세요.

여러 UDS 요청을 같은 ECU 상태에 전달하는 방법은 [Stateful 퍼징](cargo_stateful_fuzz.md)을 참고하세요.

## 왜 `lib.rs`로 분리하나?

현재 코드는 다음처럼 역할이 나뉩니다.

```text
src/
├── lib.rs   # ecu 모듈을 외부에 공개
├── main.rs  # ISO-TP 통신 프로그램
└── ecu.rs   # EcuState와 handle_request() 구현
```

`main.rs`는 실행 프로그램의 시작점입니다. 반면 cargo-fuzz 하네스는 별도의 실행 대상으로 빌드됩니다.

`main.rs` 안에만 `ecu.rs`를 연결하면 퍼징 하네스에서 동일한 코드를 가져오기 어렵습니다. `lib.rs`에서 UDS 로직을 공개하면 두 실행 대상이 같은 함수를 사용할 수 있습니다.

```text
                       ┌─ main.rs: 실제 ISO-TP 통신
lib.rs → ecu.rs ───────┤
                       └─ fuzz target: 임의 입력 테스트
```

이 구조의 장점:

- 실제 ECU와 퍼저가 동일한 `handle_request()`를 사용합니다.
- ISO-TP 장치 없이 UDS 처리 로직만 빠르게 퍼징할 수 있습니다.
- 통신 코드와 요청 처리 코드를 분리할 수 있습니다.

## 파일을 불러오는 문법

### `lib.rs`

```rust
pub mod ecu;
```

각 부분의 의미:

- `mod ecu`: 같은 `src` 폴더의 `ecu.rs`를 모듈로 연결합니다.
- `pub`: 이 모듈을 `main.rs`와 퍼징 하네스 같은 외부 코드에서 사용할 수 있게 공개합니다.
- `ecu`: 기본적으로 `src/ecu.rs`를 찾습니다.

연결 관계:

```text
src/lib.rs의 pub mod ecu;
                │
                └── src/ecu.rs
```

### `main.rs`

```rust
use uds_ecu_rust::ecu::{handle_request, EcuState};
```

각 경로의 의미:

```text
uds_ecu_rust        Cargo.toml의 패키지 이름 uds-ecu-rust
└── ecu             lib.rs에서 공개한 ecu 모듈
    ├── EcuState
    └── handle_request
```

Cargo 패키지 이름에는 하이픈이 들어갈 수 있지만, Rust 코드에서 가져올 때는 하이픈을 밑줄로 바꿉니다.

```text
Cargo.toml: uds-ecu-rust
Rust 코드:  uds_ecu_rust
```

중괄호는 같은 모듈에서 여러 항목을 한 번에 가져오는 문법입니다.

```rust
use uds_ecu_rust::ecu::{handle_request, EcuState};
```

아래처럼 따로 작성한 것과 같습니다.

```rust
use uds_ecu_rust::ecu::handle_request;
use uds_ecu_rust::ecu::EcuState;
```

## 퍼징 준비 절차

### 1. nightly Rust 설치

`cargo-fuzz`와 libFuzzer는 nightly Rust가 필요합니다.

```bash
rustup toolchain install nightly
```

설치 확인:

```bash
rustup run nightly rustc --version
```

### 2. cargo-fuzz 설치

```bash
cargo install cargo-fuzz
```

설치 확인:

```bash
cargo fuzz --help
```

### 3. 현재 버전 확인

`cargo-fuzz` 버전:

```bash
cargo fuzz --version
```

nightly Rust 버전:

```bash
rustc +nightly --version
```

### 4. 업데이트 방법

#### cargo-fuzz 업데이트

이미 설치된 `cargo-fuzz`를 최신 버전으로 다시 설치합니다.

```bash
cargo install cargo-fuzz --force
```

`--force`는 기존에 설치된 실행 파일이 있어도 다시 설치한다는 뜻입니다.

업데이트 확인:

```bash
cargo fuzz --version
```

#### nightly Rust 업데이트

nightly Rust 컴파일러와 Cargo를 업데이트합니다.

```bash
rustup update nightly
```

업데이트 확인:

```bash
rustc +nightly --version
```

`cargo-fuzz`와 nightly Rust는 서로 다른 도구이므로 각각 업데이트해야 합니다.

### 5. 프로젝트 폴더로 이동

다음 세 가지 방법 모두 `uds-ecu-rust` 폴더에서 실행합니다.

```bash
cd ecu-projects/uds-ecu-rust
```

### 방법 1: `init`으로 처음 시작

```bash
cargo +nightly fuzz init
```

`init`은 `fuzz/` 프로젝트와 기본 타깃을 함께 생성합니다.

```text
fuzz/
├── Cargo.toml
└── fuzz_targets/
    └── fuzz_target_1.rs
```

기본 타깃을 첫 번째 하네스로 사용하려면 파일명을 변경합니다.

```bash
mv fuzz/fuzz_targets/fuzz_target_1.rs fuzz/fuzz_targets/handle-request.rs
```

`fuzz/Cargo.toml`의 `[[bin]]` 설정도 같은 이름으로 변경합니다.

기존 설정:

```toml
[[bin]]
name = "fuzz_target_1"
path = "fuzz_targets/fuzz_target_1.rs"
```

변경한 설정:

```toml
[[bin]]
name = "handle-request"
path = "fuzz_targets/handle-request.rs"
```

이 방법은 `init`이 어떤 파일을 만드는지 직접 확인하며 학습할 때 적합합니다.

### 방법 2: `add`로 처음 시작

`fuzz/` 폴더가 없는 상태에서 원하는 타깃 이름을 지정합니다.

```bash
cargo +nightly fuzz add handle-request
```

`cargo-fuzz`가 필요한 환경과 `handle-request` 타깃을 함께 생성합니다.

```text
fuzz/
├── Cargo.toml
└── fuzz_targets/
    └── handle-request.rs
```

기본 이름인 `fuzz_target_1`을 바꾸는 과정이 없으므로, 타깃 하나를 빠르게 만들 때 가장 간단합니다.

### 방법 3: 기존 환경에 `add`로 타깃 추가

이미 `fuzz/`와 기존 타깃이 있는 상태에서 다른 하네스를 추가할 때도 `add`를 사용합니다.

예를 들어 SecurityAccess 전용 타깃을 추가하려면:

```bash
cargo +nightly fuzz add security-access
```

기존 타깃은 유지되고 새 타깃이 추가됩니다.

```text
fuzz_targets/
├── handle-request.rs
└── security-access.rs
```

현재 프로젝트에는 단일 요청용 `fuzz_target_1`과 Stateful 요청용 `stateful-sequence`가 이미 있으므로, 두 타깃을 사용할 때는 추가 명령을 실행하지 않아도 됩니다.

### 현재 프로젝트의 최종 구조

```text
ecu-projects/uds-ecu-rust/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── main.rs
│   └── ecu.rs
├── examples/
│   ├── replay_stateful_simple.rs
│   ├── replay_stateful_summary.rs
│   └── replay_stateful.rs
└── fuzz/
    ├── Cargo.toml
    └── fuzz_targets/
        ├── fuzz_target_1.rs
        └── stateful-sequence.rs
```

## 퍼징 하네스 작성

현재 단일 요청 하네스인 `fuzz/fuzz_targets/fuzz_target_1.rs`는 다음 형태입니다.

```rust
#![no_main]

use uds_ecu_rust::ecu::{handle_request, EcuState};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut state = EcuState::new();
    let _ = handle_request(&mut state, data);
});
```

### `#![no_main]`

일반 프로그램처럼 직접 `fn main()`을 만들지 않겠다는 뜻입니다. 프로그램 시작과 반복 실행은 libFuzzer가 담당합니다.

### `fuzz_target!`

```rust
fuzz_target!(|data: &[u8]| {
    // 테스트할 코드
});
```

libFuzzer가 만든 입력을 `data`로 받는 퍼징 진입점입니다.

- `data`: 입력 변수 이름
- `&[u8]`: 여러 바이트를 빌린 slice
- `|data: &[u8]|`: 입력 하나를 받는 Rust closure 문법
  - `Closure` = 이름 없이 바로 만드는 간단한 함수

### ECU 상태 생성

```rust
let mut state = EcuState::new();
```

`handle_request()`가 상태를 변경하므로 `state`를 `mut`로 선언합니다. 이 단순 하네스에서는 퍼징 입력마다 초기 상태를 새로 만듭니다.

### 퍼징 입력 전달

```rust
let _ = handle_request(&mut state, data);
```

- `&mut state`: 함수가 ECU 상태를 변경할 수 있도록 빌려줍니다.
- `data`: libFuzzer가 생성한 요청 바이트입니다.
- `let _ =`: 반환된 UDS 응답은 사용하지 않겠다는 뜻입니다.

응답 내용보다 panic이나 비정상 종료 발생 여부를 먼저 검사하는 기본 하네스입니다.

## 실행 방법

`ecu-projects/uds-ecu-rust`에서 실행합니다.

### 기본 이름으로 시작한 경우

`cargo fuzz init`이 만든 `fuzz_target_1`의 이름을 변경하지 않았다면 다음처럼 실행합니다.

```bash
cargo +nightly fuzz run fuzz_target_1
```

### `add`로 시작한 경우

다음 명령으로 `handle-request`를 만들었다면:

```bash
cargo +nightly fuzz add handle-request
```

같은 타깃 이름으로 실행합니다.

```bash
cargo +nightly fuzz run handle-request
```

### 타깃을 추가한 경우

기존 퍼징 환경에 다음 타깃을 추가했다면:

```bash
cargo +nightly fuzz add security-access
```

추가한 타깃 이름으로 실행합니다.

```bash
cargo +nightly fuzz run security-access
```

즉, 생성 방법과 관계없이 `cargo fuzz run` 뒤에는 실행하려는 타깃 이름을 적습니다. 중지는 `Ctrl+C`입니다.

실행 시간을 제한하려면 libFuzzer 옵션을 전달합니다.

```bash
cargo +nightly fuzz run fuzz_target_1 -- -max_total_time=60
```

위 명령은 약 60초 동안 퍼징합니다.

## 발생했던 import 오류

하네스에서 다음처럼 crate 최상위에서 항목을 가져오려고 해서 컴파일 오류가 발생했습니다.

```rust
use uds_ecu_rust::{EcuState, handle_request};
```

실제 항목은 `ecu` 모듈 안에 있으므로 다음처럼 수정했습니다.

```rust
use uds_ecu_rust::ecu::{EcuState, handle_request};
```

연결 경로는 다음과 같습니다.

```text
src/lib.rs의 pub mod ecu;
    → src/ecu.rs
        → EcuState
        → handle_request()
```

현재 기본 타깃 이름은 `fuzz_target_1`입니다. 최신 Cargo에서 kebab-case 이름을 권장하는 경고가 나올 수 있지만 퍼징 실행에는 영향을 주지 않습니다.

```bash
cargo +nightly fuzz run fuzz_target_1
```

원한다면 파일명과 `fuzz/Cargo.toml`의 `name`, `path`를 함께 수정해 `handle-request` 같은 이름으로 변경할 수 있습니다.

## 오류 입력이 발견된 경우

문제를 일으킨 입력은 일반적으로 다음 위치에 저장됩니다.

```text
fuzz/artifacts/fuzz_target_1/
```

해당 입력을 다시 실행하려면 artifact 파일 경로를 전달합니다.

```bash
cargo +nightly fuzz run fuzz_target_1 \
  fuzz/artifacts/fuzz_target_1/<artifact-file>
```

`<artifact-file>` 부분에는 실제 생성된 파일명을 사용합니다.

코드를 수정한 후 같은 입력으로 다시 실행하면 문제가 해결됐는지 확인할 수 있습니다.

## ISO-TP 테스트와 차이

```text
Docker 통합 테스트
Tester → vcan0 → ISO-TP → main.rs → handle_request()

cargo-fuzz
임의 입력 ──────────────────────→ handle_request()
```

Docker ISO-TP 테스트:

- 작성한 UDS 요청과 예상 응답을 검사합니다.
- 실제 ISO-TP 통신과 `vcan0`를 사용합니다.
- 통신과 정상 요청 흐름을 확인합니다.

cargo-fuzz:

- libFuzzer가 여러 입력을 자동으로 생성합니다.
- ISO-TP와 `vcan0`를 사용하지 않습니다.
- panic과 비정상 종료를 탐색합니다.

두 테스트는 서로 대체하지 않습니다. Docker 테스트는 정상 통신 흐름을 확인하고, cargo-fuzz는 다양한 비정상 입력을 탐색합니다.

## 현재 하네스의 한계

위 하네스는 입력 하나마다 새로운 `EcuState`를 만듭니다. 따라서 다음과 같은 여러 요청의 상태 전이는 충분히 검사하지 못합니다.

```text
10 03 → 27 01 → 27 02 B8 9E
```

처음에는 단일 요청 하네스로 `handle_request()`의 안전성을 확인하는 것이 좋습니다. 이후 필요하면 하나의 퍼징 입력을 여러 UDS 요청으로 나누어 순서대로 전달하는 상태 기반 하네스를 추가할 수 있습니다.

고정 Seed와 XOR Key는 학습용 구현입니다. 실제 ECU의 SecurityAccess 보안 방식으로 사용하면 안 됩니다.
