

{
  description = "";

  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";

    systems-default = { url = "path:./flake.systems.nix"; flake = false; }; # github:nix-systems/default + riscv
  };

  outputs = { self, nixpkgs, utils, naersk, flake-parts, systems-default, ... } @ inputs:
    flake-parts.lib.mkFlake {inherit inputs;} {
      systems = import systems-default;

      perSystem = {
        pkgs,
        system,
        ...
      }: let 
        manifest = (builtins.fromTOML (builtins.readFile ./Cargo.toml));

        overlay = final: prev: {
          "${manifest.package.name}" = final.callPackage ./. { inherit naersk; };
        };

        riscv-toolchain-pkgs = import nixpkgs {            
          localSystem = "${system}";
          crossSystem = {
            config = "riscv64-none-elf";
          };
        };
      in {

        _module.args.pkgs = import nixpkgs {
            inherit system;
            overlays = [
              overlay
            ];
          };

        packages = {
          default = riscv-toolchain-pkgs.buildPackages.gcc;
          gdb = riscv-toolchain-pkgs.buildPackages.gdb;
          "${manifest.package.name}" = pkgs."${manifest.package.name}"
        };
      };
    };
    
}


