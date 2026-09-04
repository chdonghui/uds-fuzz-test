#!/usr/bin/env bash
set -e

INTERFACE="${INTERFACE:-vcan0}"
EXPECTED="62 F1 90 01 02 03 04"
RESPONSE_FILE=/tmp/uds_response.txt

# Tester: TX 0x7E0, RX 0x7E8
timeout 5 isotprecv -s 7E0 -d 7E8 "${INTERFACE}" > "${RESPONSE_FILE}" &
RECEIVER_PID=$!

sleep 1
printf '22 F1 90\n' | isotpsend -s 7E0 -d 7E8 "${INTERFACE}"

wait "${RECEIVER_PID}"
RESPONSE="$(tr '[:lower:]' '[:upper:]' < "${RESPONSE_FILE}" | xargs)"

if [[ "${RESPONSE}" != "${EXPECTED}" ]]; then
    echo "FAIL: expected '${EXPECTED}', received '${RESPONSE}'"
    exit 1
fi

echo "PASS: ${RESPONSE}"
