{
  description = "Tool to easily archive streams with chat and metadata.";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      extraPrograms = [ "twitch_downloader" "chat_downloader" "format" ];
      forAllSystems = nixpkgs.lib.genAttrs [
        "aarch64-linux"
        "x86_64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      config = {
        permittedInsecurePackages = [
          "dotnet-runtime-6.0.36"
          "dotnet-runtime-wrapped-6.0.36"
          "dotnet-sdk-6.0.136"
          "dotnet-sdk-wrapped-6.0.136"
        ];
      };
      packageName = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.name;
      overlays = [ self.overlays.default ];
      liftPackages = pkgs: nixpkgs.lib.foldl'
        (acc: x: acc // { "${x}" = pkgs.${x}; })
        { default = pkgs.${packageName}; }
        extraPrograms;
      liftOverlays = pkgs: nixpkgs.lib.foldl'
        (acc: x: acc // { "${x}" = pkgs.callPackage nix/${x}.nix { }; })
        { ${packageName} = pkgs.callPackage ./package.nix { }; }
        extraPrograms;
    in
    {

      overlays.default = final: prev: liftOverlays final;

      packages = forAllSystems (system:
        let pkgs = import ./nixpkgs.nix { inherit system overlays config; };
        in liftPackages pkgs
      );
      legacyPackages = self.packages;

      checks = forAllSystems (system:
        let pkgs = import ./nixpkgs.nix { inherit system overlays config; };
        in import nix/checks.nix { inherit pkgs; }
      );

      apps = forAllSystems (system:
        let pkgs = import ./nixpkgs.nix { inherit system overlays config; };
        in import nix/apps.nix { inherit pkgs; }
      );

      devShells = forAllSystems (system:
        let pkgs = import ./nixpkgs.nix { inherit system overlays config; };
        in import ./shell.nix { inherit pkgs; }
      );

    };
}
