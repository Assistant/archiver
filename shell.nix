{ pkgs ? (import ./nixpkgs.nix) { }
}: {
  default = pkgs.mkShell {
    NIX_CONFIG = "experimental-features = nix-command flakes";
    nativeBuildInputs = with pkgs; [ nix git ];
    buildInputs = with pkgs; [
      rust-analyzer
      rustPlatform.rustcSrc
      rustc
      rustfmt
      cargo
      clippy

      nil
      nixpkgs-fmt

      format
    ];
  };
}
