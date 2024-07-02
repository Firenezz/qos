{ riscv-toolchain-pkgs, pkgs, system, naersk }:
let
  toolchain = (pkgs.rustChannelOf {
      rustToolchain = ./rust-toolchain.toml;
      sha256 = "sha256-X4me+hn5B6fbQGQ7DThreB/DqxexAhEQT8VNvW6Pwq4=";
      #        ^ After you run `nix build`, replace this with the actual
      #          hash from the error message
    }).rust;
  naerskLib = pkgs.callPackage naersk {
    cargo = toolchain;
    rustc = toolchain;
  };
in {
  kernel = naerskLib.buildPackage {
    name = "kernel";
    src = ./.;

    preBuild = ''
      
    '';
  };
}