{ lib
, fetchFromGitHub
, buildDotnetModule
, dotnetCorePackages
}:
buildDotnetModule rec {
  pname = "TwitchDownloader";
  version = "1.55.0";

  src = fetchFromGitHub {
    owner = "lay295";
    repo = "TwitchDownloader";
    rev = "${version}";
    sha256 = "sha256-OB11WN+oSwLCBUX2tsUN+A1Fw8zNQYCjv5eB4XkZ1jA=";
  };

  dotnet-sdk = dotnetCorePackages.sdk_6_0_1xx;

  projectFile = "TwitchDownloaderCLI/TwitchDownloaderCLI.csproj";
  nugetDeps = ./deps.nix;

  meta = with lib; {
    homepage = "https://github.com/lay295/TwitchDownloader";
    description = "Twitch VOD/Clip Downloader - Chat Download/Render/Replay";
    license = licenses.mit;
  };
}
