#!/usr/bin/env bash
# Cursor/cppdbg entrypoint: use nix gdb-riscv when riscv64-none-elf-gdb is not on PATH.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if command -v riscv64-none-elf-gdb >/dev/null 2>&1; then
  exec riscv64-none-elf-gdb "$@"
fi
exec nix --extra-experimental-features 'nix-command flakes' develop "${ROOT}#gdb-riscv" -c riscv64-none-elf-gdb "$@"
