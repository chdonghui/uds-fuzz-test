#!/usr/bin/env bash
set -e

# Extended Session 응답을 기다린다.
timeout 5 isotprecv -s 7E0 -d 7E8 vcan0 > /tmp/session_response &
PID=$!

sleep 1

# ECU를 Extended Session으로 변경한다.
echo "10 03" | isotpsend -s 7E0 -d 7E8 vcan0

wait "$PID"
grep -q "50 03" /tmp/session_response
echo "PASS session: 50 03"

# DID 응답을 기다린다.
timeout 5 isotprecv -s 7E0 -d 7E8 vcan0 > /tmp/did_response &
PID=$!

sleep 1

# Extended Session에서 DID F190을 요청한다.
echo "22 F1 90" | isotpsend -s 7E0 -d 7E8 vcan0

wait "$PID"
grep -q "62 F1 90 01 02 03 04" /tmp/did_response
echo "PASS DID: 62 F1 90 01 02 03 04"

# SecurityAccess Seed 응답을 기다린다.
timeout 5 isotprecv -s 7E0 -d 7E8 vcan0 > /tmp/seed_response &
PID=$!

sleep 1

# Level 1 Seed를 요청한다.
echo "27 01" | isotpsend -s 7E0 -d 7E8 vcan0

wait "$PID"
grep -q "67 01 12 34" /tmp/seed_response
echo "PASS seed: 67 01 12 34"

# SecurityAccess Key 응답을 기다린다.
timeout 5 isotprecv -s 7E0 -d 7E8 vcan0 > /tmp/key_response &
PID=$!

sleep 1

# 0x1234 XOR 0xAAAA로 계산한 Level 1 Key를 전송한다.
echo "27 02 B8 9E" | isotpsend -s 7E0 -d 7E8 vcan0

wait "$PID"
grep -q "67 02" /tmp/key_response
echo "PASS key: 67 02"
