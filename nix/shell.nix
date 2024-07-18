
{ riscv-pkgs, pkgs, qos-package }:
pkgs.mkShell {
  inputsFrom = [ qos-package.qos ];
  nativeBuildInputs = [
    rust-analyzer # LSP Server
  ];

  packages = [
    pkgs.qemu_full
    riscv-pkgs.OVMF.fd
    riscv-pkgs.buildPackages.gdb
  ];
}