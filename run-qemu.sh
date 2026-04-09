#!/bin/bash

# Check if disk image is provided
if [ -z "$1" ]; then
    echo "Usage: $0 <disk_image>"
    echo "Example: $0 disk.qcow2"
    exit 1
fi

DISK_IMAGE="$1"

qemu-system-riscv64 \
    -machine virt \
    -cpu rv64 \
    -smp 4 \
    -m 2G \
    -bios ovmf.fd \
    -drive file="$DISK_IMAGE",format=qcow2,if=virtio \
    -device virtio-net-device,netdev=net0 \
    -netdev user,id=net0 \
    -nographic 