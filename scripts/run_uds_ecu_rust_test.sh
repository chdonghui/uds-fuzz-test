#!/usr/bin/env bash
set -e

cd "$(dirname "$0")/.."

cleanup() {
    docker compose -f docker/uds-ecu-rust.yml down
}
trap cleanup EXIT

sudo modprobe vcan

if ! ip link show vcan0 >/dev/null 2>&1; then
    sudo ip link add dev vcan0 type vcan
fi

sudo ip link set vcan0 up

docker compose -f docker/uds-ecu-rust.yml build
docker compose -f docker/uds-ecu-rust.yml up -d uds-ecu-rust
sleep 2
docker compose -f docker/uds-ecu-rust.yml run --rm uds-ecu-rust-test
