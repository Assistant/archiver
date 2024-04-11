{ writeShellApplication
, pkgs
, cargo
, nixpkgs-fmt
}:
writeShellApplication {
  name = "format";
  runtimeInputs = with pkgs; [ rustfmt ];
  text = ''
    ${cargo}/bin/cargo fmt --manifest-path ./Cargo.toml
    ${nixpkgs-fmt}/bin/nixpkgs-fmt .
  '';
}

