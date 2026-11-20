#!/usr/bin/env just --justfile

build-dir := justfile_directory() / "build"
maps-dir := build-dir / "maps"

kernel-name := "qos-kernel"

limine-dir := justfile_directory() / "arch" / "limine"

# Matches pkgsCross.riscv64-embedded (nix develop .#gdb-riscv).
gnu-target := "riscv64-none-elf"

## BinUtils
objcopy := require(gnu-target + "-objcopy")
objdump := require(gnu-target + "-objdump")
ar := require(gnu-target + "-ar")
as := require(gnu-target + "-as")
ld := require(gnu-target + "-ld")
strip := require(gnu-target + "-strip")
readelf := require(gnu-target + "-readelf")
size := require(gnu-target + "-size")
nm := require(gnu-target + "-nm")

set dotenv-load := true

import 'arch/limine/justfile'

# QEMU devices
QEMU_DEVICE_RAMFB := "ramfb"
QEMU_DEVICE_XHCI := "qemu-xhci"
QEMU_DEVICE_GPU_PCI := "virtio-gpu-pci"
QEMU_DEVICE_NET := "-netdev user,id=net0 -device virtio-net-pci,netdev=net0"
QEMU_DEVICE_RNG := "virtio-rng-pci"

gdb-port := "3333"
kernel-elf := build-dir / (kernel-name + ".elf")
kernel-sym := build-dir / (kernel-name + ".sym")
kernel-stripped := build-dir / (kernel-name + ".stripped.elf")
kernel-map := maps-dir / (kernel-name + ".map")
kernel-local-map := maps-dir / (kernel-name + ".local.map")
kernel-memmap := maps-dir / (kernel-name + ".memmap.txt")

# Demangled symbols kept in the local-only map (workspace crates + entry).
# `$$` → `$` for the shell (just would otherwise eat `$`).
local-sym-pattern := "qos_kernel::|qos_types::|riscv_rs::|boot_main$$"

# Prepare a writable UEFI vars image (QEMU store copy is read-only).
prepare-ovmf-vars: build-folder edk2-ovmf
    cp -f "{{env("OVMF_VARS")}}" "{{build-dir}}/RISCV_VIRT_VARS.fd"
    chmod u+w "{{build-dir}}/RISCV_VIRT_VARS.fd"

run-riscv64 *QEMU_ARGS: prepare-ovmf-vars iso-build
    echo "OVMF_CODE: '{{env("OVMF_CODE")}}'"
    qemu-system-riscv64 \
        -machine virt,acpi=on \
        -m 4G \
        -cpu rv64,svpbmt=on \
        -smp 4 \
        -device {{QEMU_DEVICE_RAMFB}} \
        -device {{QEMU_DEVICE_GPU_PCI}} \
        -device {{QEMU_DEVICE_XHCI}} \
        {{QEMU_DEVICE_NET}} \
        -device {{QEMU_DEVICE_RNG}} \
        -device usb-kbd \
        -device usb-mouse \
        -serial stdio \
        -drive if=pflash,format=raw,file={{env("OVMF_CODE")}},readonly=on \
        -drive if=pflash,format=raw,file={{build-dir}}/RISCV_VIRT_VARS.fd \
        -drive file={{build-dir}}/{{kernel-name}}.iso,format=raw,id=cd0 \
        {{QEMU_ARGS}}

# Build ISO and freeze QEMU until a GDB client attaches (tcp::3333).
run-riscv64-gdb *QEMU_ARGS: prepare-ovmf-vars iso-build
    #!/usr/bin/env bash
    set -euo pipefail
    test -f "{{kernel-elf}}"
    # Marker for .vscode/tasks.json background endsPattern (after ISO/ELF exist).
    echo "GDB server listening on localhost:{{gdb-port}}"
    exec qemu-system-riscv64 \
        -machine virt,acpi=on \
        -m 4G \
        -cpu rv64,svpbmt=on \
        -smp 4 \
        -device ramfb \
        -device {{QEMU_DEVICE_GPU_PCI}} \
        -device {{QEMU_DEVICE_XHCI}} \
        {{QEMU_DEVICE_NET}} \
        -device {{QEMU_DEVICE_RNG}} \
        -device usb-kbd \
        -device usb-mouse \
        -serial stdio \
        -drive if=pflash,format=raw,file={{env("OVMF_CODE")}},readonly=on \
        -drive if=pflash,format=raw,file={{build-dir}}/RISCV_VIRT_VARS.fd \
        -drive file={{build-dir}}/{{kernel-name}}.iso,format=raw,id=cd0 \
        -S -gdb tcp::{{gdb-port}} \
        {{QEMU_ARGS}}

# CLI GDB client. Requires run-riscv64-gdb in another terminal.
# Uses the unstripped {{kernel-elf}} (DWARF kept in-tree for debugging).
gdb-attach:
    {{gnu-target}}-gdb \
        -ex "set architecture riscv:rv64" \
        -ex "target remote localhost:{{gdb-port}}" \
        {{kernel-elf}}

run-riscv64-dumpdts: prepare-ovmf-vars
    qemu-system-riscv64 \
        -machine virt,dumpdtb=qemu-new.dtb \
        -m 4G \
        -cpu rv64 \
        -smp 4 \
        -device {{QEMU_DEVICE_RAMFB}} \
        -device {{QEMU_DEVICE_GPU_PCI}} \
        -device {{QEMU_DEVICE_XHCI}} \
        {{QEMU_DEVICE_NET}} \
        -device {{QEMU_DEVICE_RNG}} \
        -device usb-kbd \
        -device usb-mouse \
        -serial mon:stdio \
        -drive if=pflash,format=raw,file={{env("OVMF_CODE")}},readonly=on \
        -drive if=pflash,format=raw,file={{build-dir}}/RISCV_VIRT_VARS.fd \
        -drive file={{build-dir}}/{{kernel-name}}.iso,format=raw,id=cd0
    dtc -I dtb -O dts qemu-new.dtb > qemu-new.dts

build-folder:
    mkdir -p "{{build-dir}}" "{{maps-dir}}"

# Compile and copy the unstripped ELF, then write ELF section memmap + linker map.
build: _kernel-elf memory-map

[private]
_kernel-elf: build-folder
    cargo build --target riscv64gc-unknown-none-elf -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
    cp -v target/riscv64gc-unknown-none-elf/debug/{{kernel-name}} {{kernel-elf}}

run:
    cargo run --target riscv64gc-unknown-none-elf --release

build-docs:
    cargo doc --target riscv64gc-unknown-none-elf --release

# Remove cargo artifacts, build outputs, and generated maps/ISO files.
clean:
    cargo clean
    rm -rf "{{build-dir}}"
    rm -f qemu-new.dtb qemu-new.dts

# Human-readable layout of the kernel ELF (sections + your crates only).
# Full linker map (all deps): {{kernel-map}}
# Filtered symbols (no core/alloc/third-party): {{kernel-local-map}}
memory-map: build-folder
    @test -f "{{kernel-elf}}" || (echo "error: missing {{kernel-elf}} — run just build first" >&2; exit 1)
    @printf '%s\n' "=== {{kernel-name}} ELF section memory map ===" "file: {{kernel-elf}}" "" > "{{kernel-memmap}}"
    {{readelf}} -S -W "{{kernel-elf}}" >> "{{kernel-memmap}}"
    @printf '%s\n' "" "=== program headers (load segments) ===" "" >> "{{kernel-memmap}}"
    {{readelf}} -l -W "{{kernel-elf}}" >> "{{kernel-memmap}}"
    @printf '%s\n' \
        "VMA              Type Symbol (workspace crates only; core/deps excluded)" \
        > "{{kernel-local-map}}"
    {{nm}} -n --demangle "{{kernel-elf}}" \
        | rg '{{local-sym-pattern}}' \
        >> "{{kernel-local-map}}"
    @printf '%s\n' "" "=== local symbols ({{kernel-local-map}}) ===" "" >> "{{kernel-memmap}}"
    @cat "{{kernel-local-map}}" >> "{{kernel-memmap}}"
    @echo "wrote {{kernel-memmap}}"
    @echo "wrote {{kernel-local-map}} (excluded core/alloc/compiler_builtins/third-party)"

# Optional side-band artifacts only — does not modify {{kernel-elf}} (ISO/debug stay unstripped).
# Produces {{kernel-name}}.sym + {{kernel-name}}.stripped.elf with .gnu_debuglink.
strip-debug: build
    {{objcopy}} --only-keep-debug {{kernel-elf}} {{kernel-sym}}
    cp -v {{kernel-elf}} {{kernel-stripped}}
    {{objcopy}} --strip-debug {{kernel-stripped}}
    cd {{build-dir}} && {{objcopy}} --add-gnu-debuglink={{kernel-name}}.sym {{kernel-name}}.stripped.elf
    @echo "symbols (side): {{kernel-sym}}"
    @echo "stripped ELF (side): {{kernel-stripped}}"
    @echo "unstripped ELF (ISO + GDB): {{kernel-elf}}"

edk2-ovmf:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -z "${OVMF_CODE:-}" || -z "${OVMF_VARS:-}" ]]; then
        echo "error: OVMF_CODE and OVMF_VARS must be set (nix develop .#gdb-riscv)" >&2
        exit 1
    fi
    if [[ ! -f "$OVMF_CODE" || ! -f "$OVMF_VARS" ]]; then
        echo "error: OVMF firmware not found:" >&2
        echo "  OVMF_CODE=$OVMF_CODE" >&2
        echo "  OVMF_VARS=$OVMF_VARS" >&2
        exit 1
    fi
    echo "OVMF_CODE=$OVMF_CODE"
    echo "OVMF_VARS=$OVMF_VARS"

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