#!/usr/bin/env just --justfile

build-dir := justfile_directory() / "build"

kernel-name := "qos-kernel"

limine-dir := justfile_directory() / "limine"

make := require("make")

import 'limine/justfile'

run-riscv64 +QEMU_ARGS: edk2-ovmf iso-build
    qemu-system-riscv64 \
        -machine virt \
        -kernel target/riscv64gc-unknown-none-elf/release/kernel \
        -m 128M \
        -nographic \
        -bios none \
        -initrd initramfs.cpio \
        -append "console=ttyS0" \
        -serial mon:stdio \
        -display none \
        {{QEMU_ARGS}}

build-folder:
    mkdir -p "{{build-dir}}"

build:
    cargo build --target riscv64gc-unknown-none-elf -Zbuild-std

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
    mkdir -p {{build-dir}}/iso_root/EFI/BOOT
    cp -v {{limine-dir}}/limine_root/* {{build-dir}}/iso_root/
    cd {{build-dir}} && cp -v {{kernel-name}}.elf \
        {{limine-dir}}/limine/limine-bios.sys {{limine-dir}}/limine/limine-bios-cd.bin {{limine-dir}}/limine/limine-uefi-cd.bin {{build-dir}}/iso_root/
    cd {{build-dir}} && cp -v {{limine-dir}}/limine/BOOTX64.EFI {{build-dir}}/iso_root/EFI/BOOT/
    cd {{build-dir}} && cp -v {{limine-dir}}/limine/BOOTIA32.EFI {{build-dir}}/iso_root/EFI/BOOT/
    cd {{build-dir}} && cp -v {{limine-dir}}/limine/BOOTRISCV64.EFI {{build-dir}}/iso_root/EFI/BOOT/
    cd {{build-dir}} && xorriso -as mkisofs -b limine-bios-cd.bin \
