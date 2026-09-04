#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

docker compose -f docker/uds-ecu-rust.yml down --rmi local --volumes --remove-orphans

if ip link show vcan0 >/dev/null 2>&1; then
    sudo ip link delete vcan0
fi

sudo modprobe -r vcan 2>/dev/null || true

echo "uds-ecu-rust 테스트 환경 정리 완료"
