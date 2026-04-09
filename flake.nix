{
  description = "qos: RISC-V OS with Limine bootloader and OVMF";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        platform = {
          hostPlatform = system;
          targetPlatform = "riscv64-linux";
        };
        pkgs = nixpkgs.legacyPackages.${system};
        riscv-toolchain-pkgs = import nixpkgs {            
          localSystem = "${system}";
          crossSystem = {
            config = "riscv64-unknown-linux-gnu";
          };
        };
      in rec
      {
        packages = {
          # RISC-V OVMF firmware for QEMU UEFI boot
          ovmf = riscv-toolchain-pkgs.OVMF.override {
            secureBoot = false;
            tpmSupport = false;
          };
          gdb = riscv-toolchain-pkgs.buildPackages.gdb;
          gcc = riscv-toolchain-pkgs.buildPackages.gcc;
          default = self.packages.${system}.ovmf;
        };

        devShells = {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustup

              # Build: Limine (host cc), kernel (cargo), ISO
              gnumake
              nasm
              acpica-tools

              # ISO / disk image
              xorriso
              mtools
              dtc

              # Run and debug (host GDB can debug RISC-V via QEMU gdbstub)
              qemu
            ];
          };
          gdb-riscv = pkgs.mkShell {
            buildInputs = [
              riscv-toolchain-pkgs.buildPackages.gcc
            ];
            packages = with pkgs; [
              riscv-toolchain-pkgs.buildPackages.gdb
              rustup

              # Build: Limine (host cc), kernel (cargo), ISO
              gnumake
              nasm
              acpica-tools

              # ISO / disk image
              xorriso
              mtools
              dtc

              # Run and debug (host GDB can debug RISC-V via QEMU gdbstub)
              qemu
            ];
            nativeBuildInputs = [
              riscv-toolchain-pkgs.OVMF
            ];
            OVMF_PATH = riscv-toolchain-pkgs.OVMF.fd;
          };
        };
        # Use rust-toolchain.toml via rustup (rustup default nightly, etc.)
        env = {
          #CARGO_BUILD_TARGET = "riscv64gc-unknown-none-elf";
          OVMF_PATH = "${self.packages.${system}.ovmf}";
        };
      });
}
