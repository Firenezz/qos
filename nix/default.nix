{ craneLib }:
let 
  tomlInfo = craneLib.crateNameFromCargoToml { cargoToml = ./Cargo.toml; }; 
  src = craneLib.cleanCargoSource ./.;
  
  commonArgs = {
    inherit (tomlInfo) pname version;
    inherit src;

    doCheck = true;

    strictDeps = true;

    buildInputs = with pkgs; [
      
    ];

    nativeBuildInputs = with pkgs; [
      xorriso
      pexpect
    ];
  };
in rec {
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  checks = {
    qos-clippy = craneLib.cargoClippy {
      inherit cargoArtifacts src;
      cargoClippyExtraArgs = "-- --deny warnings";
    };
  };

  qos = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
  })
}
