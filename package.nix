{ lib
, rustPlatform
, installShellFiles
, makeWrapper
, yt-dlp
, brotli
, twitch_downloader
, chat_downloader
}:
let manifest = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package;
in rustPlatform.buildRustPackage rec {
  pname = manifest.name;
  version = manifest.version;

  src = ./.;

  cargoLock.lockFile = ./Cargo.lock;

  nativeBuildInputs = [ installShellFiles makeWrapper ];

  postInstall = ''
    echo Installing man page
    assets_dir="$(dirname $(find -name archiver.1) | head -1)"
    installManPage $assets_dir/archiver.1
    echo Installing shell completion scripts
    installShellCompletion --cmd archiver \
      --bash $assets_dir/archiver.bash \
      --fish $assets_dir/archiver.fish \
      --zsh $assets_dir/_archiver
  '';

  postFixup = ''
    wrapProgram $out/bin/archiver \
      --set PATH ${lib.makeBinPath [ yt-dlp brotli twitch_downloader chat_downloader ]}
  '';

  meta = with lib; {
    description = manifest.description;
    homepage = manifest.homepage;
    changelog = "https://github.com/Assistant/archiver/blob/master/changelogs/v${version}.md";
    license = licenses.mit;
  };
}

