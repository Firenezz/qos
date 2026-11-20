# qos

Experimental RISC-V64 kernel in Rust. Boots via [Limine](https://github.com/limine-bootloader/limine) under UEFI on QEMU `virt` (OVMF).

Early kernel bring-up: Limine handoff, Sv48 paging helpers (`qmm`), frame allocator, early heap, and supervisor trap entry. Not a usable OS yet (no userspace, no scheduler).

## Status

| Area | State |
| --- | --- |
| Limine UEFI boot (`boot-limine`) | Working: requests, `BootInfo` handoff, BSP → `kernel_main` |
| Higher-half / HHDM | Used from Limine; offset plumbed into arch memory |
| Sv48 page tables (`qmm`) | Implemented as library + kernel wiring; kernel still largely on Limine’s map at halt |
| Frame allocator | Present: built from boot memory map |
| Early heap | Present: `linked_list_allocator` until a full kheap exists |
| Traps / interrupts | Present: direct-mode `stvec` → naked trap entry, exception dispatch |
| UART / logging | SBI console logger on early bring-up |
| SMP | Partial: Limine MP request + AP trampoline; APs halt (`kernel_secondary_main` unfinished) |
| Timer / IPI | Stubbed (SBI / Sstc backends TODO) |
| Scheduler | Not implemented |
| Userspace | Not implemented |

Honest scope: boot + memory + traps scaffolding. After init the BSP logs SATP state and halts.

## Layout

| Path | Role |
| --- | --- |
| `crates/kernel/` | `no_std` kernel + `qos-kernel` binary |
| `crates/qmm/` | Physical/virtual memory primitives (Sv48, page tables, frame areas, HHDM) |
| `crates/arch/riscv-rs/` | RISC-V CSR / instruction helpers |
| `crates/external/uefi/` | Local UEFI crate (submodule; not the primary boot path yet) |
| `arch/limine/` | Limine binary clone + ISO packaging helpers |
| `arch/linkers/` | RISC-V linker scripts |
| `justfile` | Build, ISO, QEMU, GDB |
| `flake.nix` / `devenv.nix` | Dev shells (QEMU, OVMF paths, `just`, RISC-V binutils/GDB) |

Workspace members: `crates/kernel`, `crates/qmm`, `crates/arch/riscv-rs`. Target: `riscv64gc-unknown-none-elf` (nightly from `rust-toolchain.toml`).

## Setup

Enter a Nix shell so `OVMF_CODE` / `OVMF_VARS`, `just`, QEMU, Rust nightly, and RISC-V binutils/GDB are available. Either works:

```bash
devenv shell           # devenv.nix (Rust toolchain + QEMU/OVMF/GDB)
# or with direnv: allow .envrc / enter the directory

nix develop .#gdb-riscv
# or: nix develop
```

Requires a Rust nightly with `riscv64gc-unknown-none-elf` and `rust-src` (see `rust-toolchain.toml`; devenv loads it automatically).

## Build

```bash
just build             # kernel ELF → build/qos-kernel.elf (+ memory maps)
just iso-build         # build + fetch Limine v10 binary + make ISO
```

Kernel cargo invocation (also used by `just`):

```bash
cargo build --target riscv64gc-unknown-none-elf \
  -Zbuild-std=core,alloc -Zbuild-std-features=compiler-builtins-mem
```

## Run (QEMU UEFI)

```bash
just run-riscv64       # ISO + qemu-system-riscv64 (virt, OVMF, serial stdio)
just run-riscv64-gdb   # same, stopped for GDB on tcp::3333
just gdb-attach        # riscv64-none-elf-gdb → remote target
```

Expected today: Limine handoff, arch init (memory, heap, traps), log SATP, then idle in `wfi`.

## Roadmap (near term)

1. Finish platform probe (timer / IPI) and drop the temporary `ebreak` in `kernel_main`
2. Own kernel page tables on top of Limine’s Sv48 map; grow beyond the early heap
3. Bring up secondary harts end-to-end
4. Only then: scheduling and userspace

## Notes

- Primary arch in-tree is RISC-V64; some Limine paths mention x86_64 stubs but are not the focus.
- `run-qemu.sh` is a separate disk-image helper; day-to-day flow is `just run-riscv64`.
- No license file in the repository yet.
