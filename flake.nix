{
  description = "RISC-V OVMF (Open Virtual Machine Firmware) environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages = {
          default = pkgs.OVMF.override {
            secureBoot = false;
            tpmSupport = false;
            targetArch = "riscv64";
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            gcc
            nasm
            python3
            acpica-tools
            iasl
            riscv64-embedded-gcc
          ];
        };
      });
} 