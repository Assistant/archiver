{ writeScriptBin
, cargo
, nixpkgs-fmt
}:
writeScriptBin "format" ''
  ${cargo}/bin/cargo fmt --manifest-path ./Cargo.toml
  ${nixpkgs-fmt}/bin/nixpkgs-fmt .
''

