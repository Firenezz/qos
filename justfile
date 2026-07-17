#!/usr/bin/env just --justfile

build-dir := justfile_directory() / "build"

kernel-name := "qos-kernel"

limine-dir := justfile_directory() / "limine"

make := require("make")

set dotenv-load := true

import 'limine/justfile'

# QEMU devices
QEMU_DEVICE_RAMFB := "ramfb"
QEMU_DEVICE_XHCI := "qemu-xhci"
QEMU_DEVICE_GPU_PCI := "virtio-gpu-pci"
QEMU_DEVICE_NET := "-netdev user,id=net0 -device virtio-net-pci,netdev=net0"
QEMU_DEVICE_RNG := "virtio-rng-pci"


run-riscv64 *QEMU_ARGS: edk2-ovmf iso-build
    echo "OVMF_PATH: '{{env("OVMF_PATH")}}'"
    qemu-system-riscv64 \
        -machine virt \
        -m 4G \
        -cpu rv64 \
        -smp 4 \
        -device ramfb \
        -device {{QEMU_DEVICE_GPU_PCI}} \
        -device {{QEMU_DEVICE_XHCI}} \
        {{QEMU_DEVICE_NET}} \
        -device {{QEMU_DEVICE_RNG}} \
        -device usb-kbd \
        -device usb-mouse \
        -serial mon:stdio \
        -drive if=pflash,format=raw,file={{env("OVMF_PATH")}}/FV/RISCV_VIRT_CODE.fd,readonly=on \
        -drive file={{build-dir}}/{{kernel-name}}.iso,format=raw,id=cd0 \
        {{QEMU_ARGS}}

run-riscv64-dumpdts:
    qemu-system-riscv64 \
        -machine dumpdtb=qemu-new.dtb \
        -m 4G \
        -cpu rv64 \
        -smp 4 \
        -device ramfb \
        -device {{QEMU_DEVICE_GPU_PCI}} \
        -device {{QEMU_DEVICE_XHCI}} \
        {{QEMU_DEVICE_NET}} \
        -device {{QEMU_DEVICE_RNG}} \
        -device usb-kbd \
        -device usb-mouse \
        -serial mon:stdio \
        -drive if=pflash,format=raw,file={{env("OVMF_PATH")}}/FV/RISCV_VIRT_CODE.fd,readonly=on \
        -drive file={{build-dir}}/{{kernel-name}}.iso,format=raw,id=cd0
    dtc -I dtb -O dts qemu-new.dtb > qemu-new.dts

build-folder:
    mkdir -p "{{build-dir}}"

build:
    cargo build --target riscv64gc-unknown-none-elf -Zbuild-std
    cp -v target/riscv64gc-unknown-none-elf/debug/{{kernel-name}} {{build-dir}}/{{kernel-name}}.elf

run:
    cargo run --target riscv64gc-unknown-none-elf --release

debug:
    cargo run --target riscv64gc-unknown-none-elf

edk2-ovmf:
    # if OVMF_PATH is not set, check if it is in the PATH else download it
    echo "TODO: download OVMF.fd if not in PATH"

iso-build: build-folder build limine
    cd {{build-dir}} && pwd
    rm -rf {{build-dir}}/iso_root
    rm -f {{build-dir}}/{{kernel-name}}.iso
    mkdir -p {{build-dir}}/iso_root/boot
    cp -v {{build-dir}}/{{kernel-name}}.elf {{build-dir}}/iso_root/boot/{{kernel-name}}.elf
    mkdir -p {{build-dir}}/iso_root/boot/limine
    cp -v {{limine-dir}}/limine_root/* {{build-dir}}/iso_root/boot/limine
    mkdir -p {{build-dir}}/iso_root/EFI/BOOT
    cd {{build-dir}} && cp -v \
        {{limine-dir}}/limine/limine-uefi-cd.bin {{build-dir}}/iso_root/boot/limine/
    cd {{build-dir}} && cp -v {{limine-dir}}/limine/BOOTRISCV64.EFI {{build-dir}}/iso_root/EFI/BOOT/
    cd {{build-dir}} && xorriso -as mkisofs \
        --efi-boot boot/limine/limine-uefi-cd.bin \
        -efi-boot-part --efi-boot-image --protective-msdos-label \
        iso_root -o {{kernel-name}}.iso