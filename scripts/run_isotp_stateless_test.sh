#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

cleanup() {
    docker compose -f docker/isotp-stateless.yml down
}
trap cleanup EXIT

sudo modprobe vcan

if ! ip link show vcan0 >/dev/null 2>&1; then
    sudo ip link add dev vcan0 type vcan
fi

sudo ip link set vcan0 up

docker compose -f docker/isotp-stateless.yml build
docker compose -f docker/isotp-stateless.yml up -d vecu
sleep 2
docker compose -f docker/isotp-stateless.yml run --rm isotp-stateless-test
