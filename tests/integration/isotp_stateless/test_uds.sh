#!/usr/bin/env bash

# 명령이 실패하면 스크립트를 바로 종료한다.
set -e

# ECU 응답을 최대 5초 동안 기다리고 파일에 저장한다.
# -s 7E0: 테스터가 요청을 보내는 CAN ID
# -d 7E8: 테스터가 응답을 받는 CAN ID
# &: 수신기를 백그라운드에서 실행한다.
timeout 5 isotprecv -s 7E0 -d 7E8 vcan0 > /tmp/response &

# 바로 앞에서 실행한 백그라운드 수신기의 프로세스 ID를 저장한다.
PID=$!

# 수신기가 준비될 시간을 준다.
sleep 1

# UDS ReadDataByIdentifier 요청 22 F1 90을 ECU에 보낸다.
echo "22 F1 90" | isotpsend -s 7E0 -d 7E8 vcan0

# 수신기가 응답을 받고 종료할 때까지 기다린다.
wait "$PID"

# 응답 파일에 예상한 UDS 응답이 있는지 검사한다.
# 찾지 못하면 grep이 실패하고 set -e에 의해 스크립트가 종료된다.
grep -q "62 F1 90 01 02 03 04" /tmp/response

# 모든 명령과 응답 검사가 성공한 경우에만 출력된다.
echo "PASS: 62 F1 90 01 02 03 04"
