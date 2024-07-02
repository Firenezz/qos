{
  description = "Flake for my custom os QOS";

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";

    nixpkgs-mozilla = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
    treefmt-nix.url = "github:numtide/treefmt-nix";

    systems-default = { url = "path:./flake.systems.nix"; flake = false; }; # github:nix-systems/default + riscv
  };

  outputs = { self, fenix, nixpkgs, utils, naersk, flake-parts, systems-default, nixpkgs-mozilla, ... } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import systems-default;

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }: let
        riscv-toolchain-pkgs = import pkgs {
          localSystem = "${system}";
          crossSystem = {
            config = "riscv64-none-elf";
          };
        };

        OVMF = riscv-toolchain-pkgs.callPackage ./nix/OVMF/default.nix {};

        buildInputs = [
          pkgs.qemu
          OVMF
        ];

        nativeBuildInputs = with pkgs; [
          rust-analyzer # LSP Server
          rustfmt       # Formatter
          clippy        # Linter
          cargo
          xorriso
          pexpect
          riscv-toolchain-pkgs.buildPackages.gcc
        ];

        debugBuildInputs = buildInputs ++ [
          riscv-toolchain-pkgs.buildPackages.gdb
        ];

        devshell = import ./devshell.nix { inherit riscv-toolchain-pkgs nativeBuildInputs debugBuildInputs; };
        #shell = import ./shell.nix { inherit buildInputs nativeBuildInputs ; };

        qos = pkgs.callPackage ./. { inherit riscv-toolchain-pkgs pkgs system naersk; };
      in rec {

        packages = {
          default = qos.kernel;
        };

        _module.args.pkgs = import nixpkgs {
            inherit system;
          };

        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
          };
        };
        
        devShells = {
          #default = devshell;

          gcc = pkgs.mkShell {
            buildInputs = [
              riscv-toolchain-pkgs.buildPackages.gcc
            ];
          };
        };
      };
    };
    
}


