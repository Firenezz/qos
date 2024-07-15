{ riscv-pkgs, local-pkgs, qos-package }:

pkgs.mkShell {
  inputsFrom = [ qos-package.qos ];
  nativeBuildInputs = [
    rust-analyzer # LSP Server
    riscv-pkgs.buildPackages.gdb
  ];

  OVMF_code = ${debugBuildInputs.OVMFFull.firmware}
}