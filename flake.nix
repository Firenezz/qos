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
        crossPkgs = pkgs.pkgsCross.riscv64-embedded;
      in
      {
        packages = {
          default = crossPkgs.OVMF.override {
            secureBoot = false;
            tpmSupport = false;
          };
        };

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            gcc
            nasm
            python3
            acpica-tools
            crossPkgs.stdenv.cc
          ];
        };
      });
} 