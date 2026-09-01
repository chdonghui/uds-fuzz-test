# uds-fuzz-test

기초 사용법 정리
./docs/instructions.md

libFuzzer 사용해 test-fuzz 퍼징
./docs/libFuzzer.md

compose.yml
docker test compose 파일
-> 추후 수정 및 발전 예정

third_party/iso14229
Github: https://github.com/driftregion/iso14229
submodule로 불러온 C언어로 개발된 오픈소스 차량 진단 프로토콜 라이브러리

test-fuzz/test.c
libFuzzer가 제대로 작동하는지 확인하기 위한 작은 테스트 대상 C언어 코드
