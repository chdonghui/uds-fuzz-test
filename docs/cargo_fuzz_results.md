# cargo-fuzz 결과 해석

cargo-fuzz 실행 결과를 확인하는 방법입니다. 아래 명령의 `<target-name>`은 실제 퍼징 타깃 이름입니다.

예:

```text
fuzz_target_1
handle-request
security-access
```

## 확인 순서

1. 터미널에서 종료 상태와 오류 확인
2. corpus에서 유용한 입력 확인
3. artifacts에서 문제 입력 확인
4. 필요하면 코드 커버리지 확인

## 1. 터미널 출력

실행 결과 예:

```text
#62645492 DONE cov: 94 ft: 96 corp: 21/61b
```

핵심 항목:

- `DONE`: 설정한 시간이나 횟수까지 실행 완료
- `cov`: 발견한 코드 커버리지 지점
- `ft`: libFuzzer가 구분한 실행 특징
- `corp`: 저장된 corpus 개수와 전체 크기
- `exec/s`: 초당 실행 횟수
- `rss`: 사용한 메모리

다음과 같은 오류 메시지가 있는지 확인합니다.

```text
panicked at
ERROR: AddressSanitizer
SUMMARY:
```

오류 메시지가 없고 `DONE`으로 끝났다면 해당 실행에서 발견된 크래시가 없다는 뜻입니다.

## 2. corpus 확인

위치:

```text
fuzz/corpus/<target-name>/
```

corpus는 퍼저가 실행한 모든 입력이 아닙니다. 새로운 코드 경로나 실행 특징을 발견하는 데 도움이 된 대표 입력만 저장합니다.

corpus 파일은 바이너리이므로 에디터나 `cat`으로 보면 깨져 보일 수 있습니다. 바이트 기반 입력은 16진수로 확인합니다.

```bash
for file in fuzz/corpus/<target-name>/*; do
    echo -n "$(basename "$file"): "
    xxd -p -u -c 4096 "$file" | sed 's/../& /g'
done
```

- `basename "$file"`: 폴더 경로 제거
- `-p`: 주소 문자 영역 제거, 순수 16진수만 출력
- `-u`: 16진수 알파벳 대문자로 출력
- `-c 4096`: 한 줄에 최대 4096 바이트까지 출력, libFuzzer의 최대 입력 크기

출력 예:

```text
10 01
10 03
22 F1 90
```

해석할 때는 다음을 확인합니다.

- 찾고 있던 입력 형식이 corpus에 있는가?
- 정상 입력과 비정상 입력이 어떤 비율로 남았는가?
- 해당 입력이 테스트 대상 코드의 어떤 조건과 일치하는가?

원하는 입력이 corpus에 있다면 퍼저가 해당 바이트를 생성했고, 그 입력이 커버리지나 실행 특징에 도움이 되어 저장됐다는 뜻입니다.

같은 코드 분기로 들어가는 다른 입력은 실제로 실행됐더라도 corpus에 남지 않을 수 있습니다.

## 3. artifacts 확인

위치:

```text
fuzz/artifacts/<target-name>/
```

확인:

```bash
ls -la fuzz/artifacts/<target-name>
```

artifact 파일 이름은 문제 종류와 입력 hash로 구성됩니다.

```text
crash-<hash>
timeout-<hash>
oom-<hash>
slow-unit-<hash>
```

- 폴더가 비어 있음: 저장된 문제 입력 없음
- `crash-*`: panic, 비정상 종료 또는 sanitizer 오류를 발생시킨 입력
- `timeout-*`: 제한 시간을 초과한 입력
- `oom-*`: 과도한 메모리를 사용한 입력
- `slow-unit-*`: 처리 속도가 매우 느린 입력

파일 내용은 문제를 일으킨 원본 바이너리 입력입니다. 하나의 artifact를 16진수 바이트로 확인합니다.

```bash
xxd -p -u -c 4096 \
    fuzz/artifacts/<target-name>/<artifact-file> \
    | sed 's/../& /g'
```

폴더의 모든 artifact를 확인하려면:

```bash
for file in fuzz/artifacts/<target-name>/*; do
    echo -n "$(basename "$file"): "
    xxd -p -u -c 4096 "$file" | sed 's/../& /g'
done
```

문제 입력을 같은 하네스에 다시 전달해 오류를 재현합니다.

```bash
cargo +nightly fuzz run <target-name> \
    fuzz/artifacts/<target-name>/<artifact-file>
```

artifact는 다음처럼 해석합니다.

```text
파일 이름 → 발생한 문제 종류와 입력 hash
파일 내용 → 문제를 일으킨 실제 입력 바이트
재현 명령 → 같은 문제가 다시 발생하는지 확인
```

## 4. 코드 커버리지 확인

corpus는 입력을 보여주지만 해당 입력이 정확히 어떤 코드 줄과 분기를 실행했는지는 보여주지 않습니다.

예를 들어 동일한 요청도 프로그램 상태에 따라 정상 처리 또는 거부 분기로 들어갈 수 있습니다. 특정 분기의 실행 여부는 코드 커버리지로 확인합니다.

현재 cargo-fuzz 버전에서 지원하는 명령을 먼저 확인합니다.

```bash
cargo +nightly fuzz coverage --help
```

## 결과 판단

다음 세 가지를 함께 확인합니다.

- corpus: 의미 있는 입력을 발견했는가?
- artifacts: 크래시나 비정상 동작을 일으킨 입력이 있는가?
- coverage: 목표 코드 분기까지 실행했는가?

corpus에 원하는 입력이 있고 artifacts가 비어 있다면, 퍼저가 그 입력을 발견했으며 현재까지 크래시는 발견하지 못했다는 뜻입니다.

다만 corpus에 입력이 있다는 사실만으로 응답이나 처리 결과가 올바르다고 단정할 수는 없습니다. 정확한 실행 경로는 커버리지 또는 별도의 검증 조건으로 확인해야 합니다.
