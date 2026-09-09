#!/usr/bin/env bash
set -euo pipefail

FUZZ_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$FUZZ_DIR/../.." && pwd)"
ISO14229_DIR="$PROJECT_ROOT/third_party/iso14229"
BUILD_DIR="$FUZZ_DIR/build"
CORPUS_DIR="$FUZZ_DIR/corpus"
ARTIFACT_DIR="$FUZZ_DIR/artifacts"
LOG_DIR="$FUZZ_DIR/logs"
BINARY="$BUILD_DIR/server-fuzz"
FUZZ_LOG="$LOG_DIR/fuzz.log"

mkdir -p "$BUILD_DIR" "$CORPUS_DIR" "$ARTIFACT_DIR" "$LOG_DIR"

# 정상 UDS 요청을 seed로 준비한다.
if [ ! -f "$CORPUS_DIR/diagnostic-default-session" ]; then
    printf '\020\001' > "$CORPUS_DIR/diagnostic-default-session"
fi
if [ ! -f "$CORPUS_DIR/diagnostic-extended-session" ]; then
    printf '\020\003' > "$CORPUS_DIR/diagnostic-extended-session"
fi
if [ ! -f "$CORPUS_DIR/read-data-by-identifier" ]; then
    printf '\042\361\220' > "$CORPUS_DIR/read-data-by-identifier"
fi
if [ ! -f "$CORPUS_DIR/read-dtc-by-status-mask" ]; then
    printf '\031\002\377' > "$CORPUS_DIR/read-dtc-by-status-mask"
fi

clang \
    -std=c11 \
    -Wall -Wextra -Wpedantic \
    -g -O1 \
    -DUDS_CUSTOM_MILLIS=1 \
    -fsanitize=fuzzer,address,undefined \
    -fno-omit-frame-pointer \
    -I"$ISO14229_DIR/src" \
    "$FUZZ_DIR/server_fuzz.c" \
    "$ISO14229_DIR/src/server.c" \
    "$ISO14229_DIR/src/tp.c" \
    "$ISO14229_DIR/src/util.c" \
    "$ISO14229_DIR/src/log.c" \
    -o "$BINARY"

if [ "${1:-}" = "--replay" ]; then
    if [ "$#" -ne 2 ] || [ ! -f "$2" ]; then
        echo "ERROR: replay artifact was not found: ${2:-<missing>}" >&2
        exit 1
    fi

    REPLAY_LOG="$LOG_DIR/replay.log"
    set +e
    "$BINARY" -runs=1 "$2" 2>&1 | tee "$REPLAY_LOG"
    REPLAY_STATUS=${PIPESTATUS[0]}
    set -e

    if [ "$REPLAY_STATUS" -ne 0 ]; then
        echo
        echo "PASS: the saved artifact reproduced a process failure."
        exit 0
    fi

    echo "FAIL: the artifact did not reproduce a process failure." >&2
    exit 1
fi

set +e
"$BINARY" \
    -max_total_time=30 \
    -max_len=4095 \
    -artifact_prefix="$ARTIFACT_DIR/" \
    "$CORPUS_DIR" 2>&1 | tee "$FUZZ_LOG"
FUZZ_STATUS=${PIPESTATUS[0]}
set -e

if [ "$FUZZ_STATUS" -ne 0 ]; then
    if grep -Eq "AddressSanitizer|UndefinedBehaviorSanitizer|runtime error:|Test unit written to" "$FUZZ_LOG"; then
        echo
        echo "FINDING: libFuzzer saved a failing input."
        echo "Artifacts: $ARTIFACT_DIR"
        exit 0
    fi

    echo "ERROR: libFuzzer stopped unexpectedly with status $FUZZ_STATUS." >&2
    exit "$FUZZ_STATUS"
fi

echo
echo "PASS: stateless server fuzzing completed without a sanitizer finding."
