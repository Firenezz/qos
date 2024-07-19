
{ riscv-pkgs, pkgs, qos-package }:
pkgs.mkShell {
  inputsFrom = [ qos-package.qos ];
  nativeBuildInputs = [
    pkgs.rust-analyzer # LSP Server
  ];

  packages = [
    pkgs.qemu_full
    riscv-pkgs.OVMF.fd
    riscv-pkgs.buildPackages.gdb
  ];
}