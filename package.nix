{ lib
, rustPlatform
, installShellFiles
, makeWrapper
, yt-dlp
, brotli
, twitch_downloader
, chat_downloader
, twitch-chat-downloader
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
    extra=$out/share/extra_completions
    mkdir -p $extra/elvish  $extra/fig $extra/powershell
    cp $assets_dir/archiver.elv $out/share/extra_completions/elvish
    cp $assets_dir/archiver.ts $out/share/extra_completions/fig
    cp $assets_dir/_archiver.ps1 $out/share/extra_completions/powershell
  '';

  postFixup = ''
    wrapProgram $out/bin/archiver \
      --set PATH ${lib.makeBinPath [ yt-dlp brotli twitch_downloader chat_downloader twitch-chat-downloader ]}
  '';

  meta = with lib; {
    description = manifest.description;
    homepage = manifest.homepage;
    changelog = "https://github.com/Assistant/archiver/blob/master/changelogs/v${version}.md";
    license = licenses.mit;
  };
}

