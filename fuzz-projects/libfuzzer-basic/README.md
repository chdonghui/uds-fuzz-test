# libFuzzer Basic

libFuzzer가 `UDS` 문자열을 찾아 의도적인 크래시를 발생시키는지 확인하는 최소 예제입니다.

## 실행

저장소 루트에서:

```bash
./fuzz-projects/libfuzzer-basic/run.sh
```

스크립트가 Docker 이미지와 실행 파일을 만들고 최대 10초 동안 퍼징합니다.

## 성공 결과

```text
PASS: libFuzzer found the UDS crash input.
```

발견한 입력은 다음 파일에 저장됩니다.

```text
fuzz-projects/libfuzzer-basic/artifacts/crash
```

바이트 확인:

```bash
xxd -g 1 fuzz-projects/libfuzzer-basic/artifacts/crash
```

확인된 값:

```text
00000000: 55 44 53  UDS
```

예제는 조건을 만족하면 `abort()`를 호출합니다. libFuzzer가 기본 처리하는 `SIGABRT`를 사용해야 ARM64에서도 오류 입력을 artifact로 저장할 수 있습니다.

`build/`, `artifacts/`, `logs/`는 실행 결과이므로 Git에서 제외됩니다.
