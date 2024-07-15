{
  description = "Flake for my custom os QOS";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs.follows = "nixpkgs";

    crane.url = "github:ipetkov/crane";
    crane.inputs.nixpkgs.follows = "nixpkgs";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    treefmt-nix.url = "github:numtide/treefmt-nix";

    systems-default = { url = "path:flake.systems.nix"; flake = false; }; # github:nix-systems/default + riscv
  };

  outputs = { self, nixpkgs, flake-parts, systems-default, ... } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import systems-default;

      imports = [
        inputs.treefmt-nix.flakeModule
      ];

      perSystem = {
        pkgs,
        system,
        ...
      }:
      let

        local-pkgs = import nixpkgs {
          localSystem = "${system}";
          #overlays = [ (import inputs.rust-overlay) ];
        };
        riscv-pkgs = import nixpkgs {
          localSystem = "${system}";
          crossSystem = {
            config = "riscv64-unknown-linux-gnu";
            abi = "lp64";
          };
        };

        rustToolchainFor = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" ];
          targets = [ "riscv64gc-unknown-none-elf" ];
        });
        rustToolchain = rustToolchainFor local-pkgs;

        craneLib = (inputs.crane.mkLib local-pkgs).overrideToolchain rustToolchainFor;

        buildInputs = [
          pkgs.qemu
          pkgs.OVMFFull
        ];

        nativeBuildInputs = with pkgs; [
          rust-analyzer # LSP Server
          rustfmt       # Formatter
          clippy        # Linter
          cargo
          xorriso
          pexpect
          riscv-pkgs.buildPackages.gcc
        ];

        debugBuildInputs = buildInputs ++ [
          riscv-pkgs.buildPackages.gdb
        ];

        qos-package = pkgs.callPackage ./nix/default.nix { inherit craneLib ;};

        #devshell = import ./devshell.nix { inherit riscv-pkgs local-pkgs qos-package; };
      in rec {

        qos = qos-package.qos;
        #qos-checks = qos-package.checks;

        packages = {
          default = qos;
        };

        treefmt.config = {
          projectRootFile = "flake.nix";
          programs = {
            nixpkgs-fmt.enable = true;
          };
        };

        checks = {
          #inherit
           # Build the crate as part of `nix flake check` for convenience
           #qos
           #qos-checks;
        };
        
        devShells = {
          default = pkgs.mkShell {
            buildInputs = [
              riscv-pkgs.buildPackages.gcc
            ];
          };

          gcc = pkgs.mkShell {
            buildInputs = [
              riscv-pkgs.buildPackages.gcc
            ];
          };
        };
      };
    };
    
}


