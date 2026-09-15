# Stateless ECU NRC semantic fuzzer

기존 `../fuzz/`는 ASan/UBSan과 응답 버퍼 경계를 확인합니다. 이 폴더는 요청별 기대 응답을 독립 oracle로 정의해 NRC와 Positive Response가 정확한지 검사합니다.

## 검사 규칙

```text
빈 요청        → 응답 없음
0x22 이외 SID  → 7F [SID] 11
잘못된 0x22 길이 → 7F 22 13
지원하지 않는 DID → 7F 22 31
22 F1 90       → 62 F1 90 01
```

## 실행

`c` 폴더에서:

```bash
./fuzz-nrc/run.sh
```

구현 응답이 oracle과 다르면 `abort()`가 호출되고 최소 입력이 `artifacts/`에 저장됩니다.
