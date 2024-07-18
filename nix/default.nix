{ craneLib, pkgs, advisory-db }:
let  
  src = craneLib.cleanCargoSource ./.;
  
  commonArgs = {
    inherit (craneLib.crateNameFromCargoToml { inherit src; }) pname version;
    inherit src;

    doCheck = true;

    strictDeps = true;

    buildInputs = with pkgs; [
      
    ];

    nativeBuildInputs = with pkgs; [
      xorriso
      #pexpect
    ];
  };
in rec {
  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  checks = {
    qos-workspace-clippy = craneLib.cargoClippy {
      inherit cargoArtifacts src;
      cargoClippyExtraArgs = "--verbose --all-targets -- --deny warnings";
    };

    # Check formatting
    qos-workspace-fmt = craneLib.cargoFmt {
      inherit src;
    };

    # Audit dependencies
    qos-workspace-audit = craneLib.cargoAudit {
      inherit src advisory-db;
    };
  };

  qos = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
  });
}
