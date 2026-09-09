# Fuzz Projects

이 폴더는 독립적으로 빌드하고 실행하는 C/C++ 퍼징 하네스를 대상별로 관리합니다.

Rust의 `cargo fuzz`가 생성하는 `ecu-projects/uds-ecu-rust/fuzz/`와 보안 재현을 격리하는 `security-analysis/iso14229/fuzz/`는 이동하지 않습니다.

## 구조

```text
fuzz-projects/
├── README.md
├── libfuzzer-basic/
│   ├── README.md
│   ├── Dockerfile
│   ├── libfuzzer_basic.c
│   ├── run.sh
│   └── run_fuzzer.sh
└── iso14229-server/
    ├── README.md
    ├── Dockerfile
    ├── server_fuzz.c
    ├── run.sh
    ├── run_fuzzer.sh
    └── replay.sh
```

## 폴더 역할

- `libfuzzer-basic/`: `UDS` 문자열을 찾아 의도적인 크래시를 발생시키는 libFuzzer 기초 예제. `run.sh`로 자동 실행
- `iso14229-server/`: raw UDS 요청 하나를 `iso14229` 서버에 전달하는 stateless 하네스. `run.sh`로 Docker 퍼징을 자동 실행

향후 서버 경계 검사와 Stateful 퍼저가 필요하면 서로 섞지 않고 다음처럼 별도 프로젝트로 추가합니다.

```text
fuzz-projects/iso14229-server-boundary/
fuzz-projects/iso14229-server-stateful/
```

각 프로젝트의 corpus, artifact, 빌드 결과와 실행 스크립트도 해당 프로젝트 폴더 안에서 관리합니다.
