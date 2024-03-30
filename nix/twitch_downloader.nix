{ lib
, fetchFromGitHub
, buildDotnetModule
}:
buildDotnetModule rec {
  pname = "twitch_downloader";
  version = "1.54.1";

  src = fetchFromGitHub {
    owner = "lay295";
    repo = "TwitchDownloader";
    rev = "${version}";
    sha256 = "sha256-Itbfx3NUhUaku4TDw3k+ykX/Ue7+4qNXNu23hejGZyk=";
  };

  projectFile = "TwitchDownloaderCLI/TwitchDownloaderCLI.csproj";
  nugetDeps = ./deps.nix;

  meta = with lib; {
    homepage = "https://github.com/lay295/TwitchDownloader";
    description = "Twitch VOD/Clip Downloader - Chat Download/Render/Replay";
    license = licenses.mit;
  };
}
