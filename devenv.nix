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
    # Targets/components come from rust-toolchain.toml; devenv rejects
    # toolchainFile combined with a manual `targets` list.
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
