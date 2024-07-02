{ pkgs, nativeBuildInputs, debugBuildInputs }:

pkgs.mkShell {
  nativeBuildInputs = nativeBuildInputs;
  buildInputs = debugBuildInputs ++ [
    make
  ];
}