{ pkgs, ... }:

let
  riscv-pkgs = pkgs.pkgsCross.riscv64-embedded;
  # QEMU ships prebuilt RISC-V EDK2. Cross-building pkgsCross...OVMF from
  # nixos-unstable currently fails (edk2-202605), so do not depend on it.
  ovmfCode = "${pkgs.qemu}/share/qemu/edk2-riscv-code.fd";
  ovmfVars = "${pkgs.qemu}/share/qemu/edk2-riscv-vars.fd";
in
{
  languages.rust = {
    enable = true;
    toolchainFile = ./rust-toolchain.toml;
    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-src"
      "llvm-tools"
      "rust-analyzer"
    ];
    targets = [ "riscv64gc-unknown-none-elf" ];
  };

  packages = with pkgs; [
    just
    just-lsp
    gnumake
    nasm
    acpica-tools
    xorriso
    mtools
    dtc
    qemu
    ripgrep
    riscv-pkgs.buildPackages.gcc
    riscv-pkgs.buildPackages.gdb
  ];

  env = {
    OVMF_CODE = ovmfCode;
    OVMF_VARS = ovmfVars;
  };
}
