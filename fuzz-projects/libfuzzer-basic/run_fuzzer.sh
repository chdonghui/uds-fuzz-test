#!/usr/bin/env bash
set -euo pipefail

EXAMPLE_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$EXAMPLE_DIR/build"
ARTIFACT_DIR="$EXAMPLE_DIR/artifacts"
LOG_DIR="$EXAMPLE_DIR/logs"
BINARY="$BUILD_DIR/libfuzzer-basic"
ARTIFACT="$ARTIFACT_DIR/crash"
LOG_FILE="$LOG_DIR/fuzz.log"

mkdir -p "$BUILD_DIR" "$ARTIFACT_DIR" "$LOG_DIR"
rm -f "$ARTIFACT"

clang \
    -std=c11 \
    -Wall -Wextra -Wpedantic \
    -g -O1 \
    -fsanitize=fuzzer,address,undefined \
    -fno-omit-frame-pointer \
    "$EXAMPLE_DIR/libfuzzer_basic.c" \
    -o "$BINARY"

set +e
"$BINARY" \
    -max_total_time=10 \
    -max_len=3 \
    -exact_artifact_path="$ARTIFACT" 2>&1 | tee "$LOG_FILE"
FUZZ_STATUS=${PIPESTATUS[0]}
set -e

if [ "$FUZZ_STATUS" -ne 0 ] && [ -f "$ARTIFACT" ] && \
   grep -q "Test unit written to" "$LOG_FILE"; then
    echo
    echo "PASS: libFuzzer found the UDS crash input."
    echo "Artifact: $ARTIFACT"
    exit 0
fi

echo "FAIL: libFuzzer did not find the expected UDS input." >&2
exit 1
