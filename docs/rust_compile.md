## cargo 프로젝트 생성

```bash
cargo new <폴더명>
cd <폴더명>
```

## 짧은 예제 모음

- ecu 폴더 안에 examples 폴더 생성 후 rs 코드 작성
- 파일 개별 실행

```bash
mkdir examples
cargo run --example <개별파일명>
```

## 공통으로 사용하는 타입이나 함수 생성 시

- src/lib.rs에 넣고 예제에서 가져오기
- lib.rs => pub struct <...> { }
- 각 예제에서 가져오기 => use ecu_demo::<...>;

## 독립된 프로그램으로 취급

- src/bin에 rs 파일 생성

```bash
cargo run -bin <파일명>
```
