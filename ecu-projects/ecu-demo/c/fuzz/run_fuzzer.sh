#!/usr/bin/env bash
set -euo pipefail

FUZZ_DIR="/work/fuzz"
BUILD_DIR="$FUZZ_DIR/build"
CORPUS_DIR="$FUZZ_DIR/corpus"
ARTIFACT_DIR="$FUZZ_DIR/artifacts"
LOG_DIR="$FUZZ_DIR/logs"
BINARY="$BUILD_DIR/stateless-ecu-fuzz"
LOG_FILE="$LOG_DIR/fuzz.log"

mkdir -p "$BUILD_DIR" "$CORPUS_DIR" "$ARTIFACT_DIR" "$LOG_DIR"

if [ ! -f "$CORPUS_DIR/read-vin" ]; then
    printf '\042\361\220' > "$CORPUS_DIR/read-vin"
fi

clang \
    -std=c11 \
    -Wall -Wextra -Wpedantic \
    -g -O1 \
    -fsanitize=fuzzer,address,undefined \
    -fno-omit-frame-pointer \
    "$FUZZ_DIR/fuzz_target.c" \
    -o "$BINARY"

"$BINARY" \
    -max_total_time=10 \
    -max_len=64 \
    -artifact_prefix="$ARTIFACT_DIR/" \
    "$CORPUS_DIR" 2>&1 | tee "$LOG_FILE"

echo
echo "PASS: stateless ECU MVP fuzzing completed."
echo "Log: $LOG_FILE"
