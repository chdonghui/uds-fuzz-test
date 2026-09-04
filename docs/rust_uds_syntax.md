# UDS 코드에서 사용하는 Rust 문법

`ecu-projects/uds-ecu-rust/src/ecu.rs`의 UDS 요청 처리 코드에 사용된 Rust 문법을 설명합니다.

## `EcuState` 구조체

```rust
pub struct EcuState {
    pub session: u8,
    pub seed: Option<u16>,
    pub security_unlocked: bool,
}
```

구조체는 관련된 값을 하나로 묶는 타입입니다.

```text
EcuState
├── session
├── seed
└── security_unlocked
```

- `session`: 현재 진단 세션
- `seed`: SecurityAccess에서 발급한 Seed
- `security_unlocked`: 보안 잠금 해제 여부

`pub`은 다른 모듈에서도 구조체나 필드에 접근할 수 있게 공개한다는 뜻입니다.

## 구조체 생성과 `Self`

```rust
impl EcuState {
    pub fn new() -> Self {
        Self {
            session: 0x01,
            seed: None,
            security_unlocked: false,
        }
    }
}
```

`impl EcuState`는 `EcuState`와 관련된 함수를 정의하는 영역입니다.

```rust
impl EcuState {
    pub fn new() -> EcuState {
        return EcuState {
            session: 0x01,
            seed: None,
            security_unlocked: false,
        };
    }
}
```

`Self`를 사용하지 않고 `EcuState`와 `return`을 사용해서 작성하는 것도 가능합니다.

```rust
EcuState::new()
```

`Self`는 현재 구현 중인 타입인 `EcuState`를 의미합니다.

```rust
Self {
    session: 0x01,
    seed: None,
    security_unlocked: false,
}
```

은 새로운 `EcuState` 값을 만듭니다.

## 구조체 필드 접근

`.`을 사용해 구조체 안의 필드에 접근합니다.

```rust
state.session
state.seed
state.security_unlocked
```

값을 읽을 수도 있고:

```rust
if state.session == 0x03 {
    // Extended Session
}
```

값을 변경할 수도 있습니다.

```rust
state.session = 0x03;
state.security_unlocked = true;
```

## `&mut EcuState`

```rust
pub fn handle_request(state: &mut EcuState, req: &[u8]) -> Vec<u8>
```

`&mut EcuState`는 기존 `EcuState`를 빌리지면서 함수 안에서 값을 변경할 수 있다는 뜻입니다.

```text
main.rs의 state
       │
       │ &mut로 빌려줌
       ▼
handle_request()
       │
       └── session, seed, security_unlocked 변경
```

함수 호출:

```rust
let response = handle_request(&mut state, &req);
```

`&mut state`를 전달했기 때문에 `handle_request()`에서 변경한 상태가 다음 요청에도 유지됩니다.

## `Option<u16>`

Seed는 항상 존재하는 값이 아닙니다.

```rust
pub seed: Option<u16>
```

`Option<T>`는 값이 있거나 없을 수 있음을 표현합니다.

```rust
None
```

Seed가 없는 상태입니다.

```rust
Some(0x1234)
```

Seed `0x1234`가 있는 상태입니다.

초기 상태:

```rust
seed: None
```

Seed 발급 후:

```rust
state.seed = Some(0x1234);
```

Key 검증이 성공한 후:

```rust
state.seed = None;
```

## `Some(seed)`

```rust
state.seed = Some(seed);
```

여기서 바깥의 `Some`은 Seed가 존재한다는 상태를 나타내고, 괄호 안의 `seed`는 실제 값입니다.

```text
Some(seed)
 │     └── 실제 값 0x1234
 └──────── 값이 있음
```

`Option<u16>`에는 `u16`을 바로 저장할 수 없습니다.

```rust
state.seed = seed;       // 타입이 달라서 불가능
state.seed = Some(seed); // 가능
```

## `let Some(seed) ... else`

```rust
let Some(seed) = state.seed else {
    return vec![0x7F, 0x27, 0x24];
};
```

다음처럼 읽을 수 있습니다.

```text
state.seed가 Some이면
→ 안에 있는 값을 seed 변수로 꺼냄

state.seed가 None이면
→ else 실행
→ 7F 27 24 반환
```

같은 동작을 `match`로 작성하면 다음과 같습니다.

```rust
let seed = match state.seed {
    Some(value) => value,
    None => return vec![0x7F, 0x27, 0x24],
};
```

`0x24`는 Seed 요청 없이 Key를 보낸 경우 사용하는 `RequestSequenceError`입니다.

## Slice 패턴 매칭

UDS 요청은 바이트 slice로 전달됩니다.

```rust
req: &[u8]
```

다음 패턴은 정확히 4바이트인 Key 요청과 일치합니다.

```rust
[0x27, 0x02, key_high, key_low]
```

예를 들어 요청이 다음과 같다면:

```text
27 02 B8 9E
```

각 변수에는 다음 바이트가 연결됩니다.

```text
key_high → B8
key_low  → 9E
```

`..`은 나머지 바이트를 의미합니다.

```rust
[0x27, ..]
```

이는 `0x27`로 시작하는 다양한 길이의 요청과 일치합니다.

## Match guard

```rust
[0x27, 0x01] if state.session != 0x03 => {
    vec![0x7F, 0x27, 0x22]
}
```

`if state.session != 0x03` 부분을 match guard라고 합니다.

요청 패턴과 상태 조건이 모두 맞아야 해당 코드가 실행됩니다.

```text
요청이 27 01인가?
AND
현재 세션이 Extended가 아닌가?
```

## `to_be_bytes()`

```rust
let seed: u16 = 0x1234;
let [high, low] = seed.to_be_bytes();
```

`u16` 값 하나를 2개의 `u8` 바이트로 나눕니다.

```text
0x1234 → [0x12, 0x34]
```

`be`는 Big Endian을 의미합니다. 높은 바이트를 먼저 배치합니다.

```text
high = 0x12
low  = 0x34
```

ISO-TP 응답은 바이트 목록이어야 하므로 Seed를 나눠 사용합니다.

```rust
vec![0x67, 0x01, high, low]
```

결과:

```text
67 01 12 34
```

## `u16::from_be_bytes()`

`from_be_bytes()`는 반대로 2개의 바이트를 `u16` 값으로 합칩니다.

```rust
let received_key = u16::from_be_bytes([
    *key_high,
    *key_low,
]);
```

```text
[0xB8, 0x9E] → 0xB89E
```

정리하면:

```text
to_be_bytes()
u16 → [u8; 2]

from_be_bytes()
[u8; 2] → u16
```

## `*key_high`의 `*`

`req`가 `&[u8]`이므로 패턴에서 꺼낸 `key_high`와 `key_low`는 바이트를 가리키는 참조입니다.

```text
key_high  : &u8
*key_high : u8
```

`u16::from_be_bytes()`는 실제 `u8` 두 개를 요구하므로 `*`로 참조 안의 값을 꺼냅니다.

```rust
u16::from_be_bytes([*key_high, *key_low])
```

이 동작을 역참조라고 합니다.

## `vec![]`와 `Vec<u8>`

```rust
vec![0x67, 0x02]
```

`vec![]`는 여러 값을 저장할 수 있는 `Vec`을 만드는 매크로입니다.

현재 함수의 반환 타입은 다음과 같습니다.

```rust
Vec<u8>
```

따라서 UDS 응답을 바이트 목록으로 반환합니다.

```rust
vec![0x67, 0x02]
```

```text
67 02
```

## `return`

```rust
return vec![0x7F, 0x27, 0x24];
```

`return`은 현재 함수를 즉시 끝내고 값을 반환합니다.

Seed가 없으면 이후 Key 계산 코드를 실행하지 않고 Negative Response를 반환합니다.

Rust에서는 match arm의 마지막 표현식도 자동으로 반환값이 됩니다.

```rust
[0x27, 0x01] => {
    vec![0x67, 0x01, high, low]
}
```

마지막 `vec![]`에는 세미콜론이 없으므로 해당 match arm의 결과가 됩니다.

## XOR Key 계산

```rust
fn calculate_key(seed: u16) -> u16 {
    seed ^ 0xAAAA
}
```

`^`는 비트 XOR 연산자입니다.

```text
0x1234 XOR 0xAAAA = 0xB89E
```

현재 XOR 방식은 SecurityAccess 흐름을 학습하기 위한 예제입니다. 실제 ECU의 보안 알고리즘으로 사용하면 안 됩니다.

## 전체 `0x27` 흐름

```text
1. 27 01 요청
2. state.seed = Some(0x1234)
3. 67 01 12 34 응답
4. 27 02 B8 9E 요청
5. let Some(seed)로 저장된 Seed 꺼내기
6. from_be_bytes()로 Key 바이트를 u16으로 변환
7. calculate_key(seed)와 비교
8. 성공하면 security_unlocked = true
9. 67 02 응답
```

관련 기능과 통신 방법은 [uds-ecu-rust](uds_ecu_rust.md)를 참고하세요.
