{
  description = "Flake for my custom os QOS";

  inputs = {
    demel-nixpkgs.url = "github:Firenezz/nixpkgs";
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
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs = { self, demel-nixpkgs, nixpkgs, flake-parts, systems-default, advisory-db, ... } @ inputs:

    flake-parts.lib.mkFlake {inherit inputs;} {
      imports = [
        inputs.treefmt-nix.flakeModule
      ];
      systems = import systems-default;

      perSystem = {
        system,
        ...
      }:
      let

        pkgs = import nixpkgs {
          localSystem = "${system}";
        };

        riscv-pkgs = import demel-nixpkgs {
          system = "${system}";
          crossSystem = {
            config = "riscv64-linux";
            #abi = "lp64";
          };
        };

        rustToolchainFor = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" ];
          targets = [ "riscv64gc-unknown-none-elf" ];
        });
        rustToolchain = rustToolchainFor pkgs;

        craneLib = (inputs.crane.mkLib pkgs).overrideToolchain rustToolchainFor;

        qos-package = pkgs.callPackage ./nix/default.nix { inherit craneLib pkgs advisory-db ;};
        qos = qos-package.qos;

        devshell = import ./nix/shell.nix { inherit riscv-pkgs pkgs qos-package; };
      in rec {

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


