{
  description = "qos: RISC-V OS with Limine bootloader and OVMF";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        riscv-pkgs = pkgs.pkgsCross.riscv64-embedded;
        # QEMU ships prebuilt RISC-V EDK2. Cross-building pkgsCross...OVMF from
        # nixos-unstable currently fails (edk2-202605), so do not depend on it.
        ovmfCode = "${pkgs.qemu}/share/qemu/edk2-riscv-code.fd";
        ovmfVars = "${pkgs.qemu}/share/qemu/edk2-riscv-vars.fd";
        commonPackages = with pkgs; [
          rustup
          just
          just-lsp
          gnumake
          nasm
          acpica-tools
          xorriso
          mtools
          dtc
          qemu
        ];
      in
      {
        packages = {
          gdb = riscv-pkgs.buildPackages.gdb;
          gcc = riscv-pkgs.buildPackages.gcc;
          default = self.packages.${system}.gdb;
        };

        devShells = {
          default = pkgs.mkShell {
            packages = commonPackages;
            OVMF_CODE = ovmfCode;
            OVMF_VARS = ovmfVars;
          };
          gdb-riscv = pkgs.mkShell {
            buildInputs = [
              riscv-pkgs.buildPackages.gcc
            ];
            packages = commonPackages ++ [
              riscv-pkgs.buildPackages.gdb
            ];
            OVMF_CODE = ovmfCode;
            OVMF_VARS = ovmfVars;
          };
        };
      });
}
