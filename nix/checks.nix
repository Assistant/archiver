{ pkgs
}: {
  format = pkgs.runCommand "check-format" { buildInputs = with pkgs; [ rustfmt cargo ]; } ''
    ${pkgs.cargo}/bin/cargo fmt --manifest-path ${../.}/Cargo.toml -- --check
    ${pkgs.nixpkgs-fmt}/bin/nixpkgs-fmt --check ${../.}
    touch $out # it worked!
  '';
}
