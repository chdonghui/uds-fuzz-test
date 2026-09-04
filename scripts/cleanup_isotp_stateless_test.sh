#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

if command -v docker >/dev/null 2>&1; then
    docker compose -f docker/isotp-stateless.yml down --rmi local --volumes --remove-orphans
fi

if ip link show vcan0 >/dev/null 2>&1; then
    sudo ip link delete vcan0
fi

sudo modprobe -r vcan 2>/dev/null || true

echo "ISO-TP 테스트 환경 정리 완료"
