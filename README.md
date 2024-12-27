# archiver
### Tool to easily archive streams with chat and metadata.

* [Configuration](#configuration)
* [Usage](#usage)
* [Installation](#installation)
  * [Shell Complete](#install-shell-autocompletion-optional)
* [Dependencies](#runtime-dependencies)
  * [Runtime](#runtime-dependencies)
  * [Build](#build-dependencies)


## Configuration
You will need to add your API tokens for Twitch and YouTube in the configuration file to archive from those sites respectively.
|OS|Configuration file location|
|---|---|
|Linux|`~/.config/archiver/config.toml`|
|Windows|`%AppData%\Assistant\archiver\config.toml`|
|macOS|`~/Library/Application Support/moe.Assistant.archiver/config.toml`|

If you run the program without one existing, a configuration file will be created, which you will need to fill out.
```toml
twitch_client_id = ""
twitch_secret = ""
youtube_key = ""
```
[Get your Twitch `Client ID` and `Secret`](https://dev.twitch.tv/docs/authentication/register-app), if you don't already have them.

[Get your YouTube `API Key`](https://developers.google.com/youtube/registering_an_application), if you don't already have one, and make sure to enable access to the `YouTube Data API v3`, and that you follow the instructions for an `API Key`, and **NOT** `OAuth 2.0`.

## Usage
`archiver [OPTIONS] <TYPE> <INPUT>`

<details>
<summary>Examples</summary>

```shell
# Download all Twitch VODs using a username
archiver --vods --channel lilyhops

# Download a specific Twitch clip using an ID
archiver --clips SpotlessKawaiiBorkArgieB8-S18P4YmbiK7gEuqG

# Download a list of Twitch Highlights containing IDs, but URLs or a combination would work too
archiver --highlights 1119099617,984635610

# Download a youtube video using a URL
archiver --youtube 'https://www.youtube.com/watch?v=11NHmPa5Ym0'
```
</details>

#### \<TYPE>
This required flag specifies the type of video to archive, you must have one and only one of the following
- `--clips`: Archive Twitch Clips
- `--highlights`: Archive Twitch Highlights
- `--vods`: Archive Twitch VODs
- `--youtube`: Archive YouTube videos

#### \<INPUT>
This required option specifies which videos to archive, you must have one and only one of the following
- `-c <CHANNEL>`, `--channel <CHANNEL>`: Archive all videos from `<CHANNEL>`
- `<VIDEOS>`: A single or list of videos to archive

`<CHANNEL>` may be an ID or username.

`<VIDEOS>` may be an ID, URL of a video, or a comma separated list.

#### [OPTIONS]
These are optional flags that affect how the program works.
- `-h`, `--help`: Print help information [does not require `<TYPE>` or `<INPUT>`]
- `-K`, `--skip-video`: Skip video download
- `-l`, `--logging`: Enable logging of external commands, e.g., `yt-dlp` will create `<id>.video.log` with its output
- `-N <THREADS>`, `--threads <THREADS>`: Number of video pieces to download simultaneously [default: 1]
- `-s`, `--silent`: Suppress output, using it twice will suppress errors too
- `-v`, `--verbose`: Increases output, useful for debugging and reporting issues
- `-V`, `--version`: Print version information [does not require `<TYPE>` or `<INPUT>`]

These optional flags are only used when downloading Twitch Clips with both `--clips` and `--channel` options.
- `-i <DURATION>`, `--interval <DURATION>`: Time interval to search for clips from a channel, shorter intervals will take longer but produce more complete results [default: `1hour`]
- `-r <DURATION>`, `--range <DURATION>`: How long ago to start searching for clips [default: `1week`]

`<DURATION>` is a string containing numbers followed by a unit, which can be follow by another number and unit to add them together.

|| Units ||
|---|---|---|
|`seconds`|`second`|`s`|
|`minutes`|`minute`|`m`|
|`hours`|`hour`|`h`|
|`days`|`day`|`d`|
|`weeks`|`week`|`w`|

Example: `1week3days12h30m45s`

## Installation
[Download](https://github.com/Assistant/archiver/releases/latest) the appropriate executable for your platform or compile it from source using the following instructions. If downloading it remember to rename it to `archiver`/`archiver.exe` for convenience.
<details>
<summary>Compiling from source</summary>

  ```shell
  git clone https://github.com/Assistant/archiver
  cd archiver
  cargo build --release
  ```
  Executable will be found at `target/release/archiver`.
</details>

Copy the executable into a directory within your `$PATH` (e.g., `~/.local/bin/`, `/usr/local/bin/`, etc.).

#### Install shell autocompletion [Optional]
  
The `build.rs` script will generate a man file (`archiver.1`) and completion scripts for all supported shells.
Supported shells are `bash`, `elvish`, `fig`, `fish`, `powershell`, and `zsh`.

## Runtime Dependencies
#### These programs need to be installed and in your path for every feature to work.
* [`yt-dlp`](https://github.com/yt-dlp/yt-dlp): Downloads the video files.
* [`TwitchChatDownloader`](https://github.com/TheDrHax/Twitch-Chat-Downloader): Downloads chat for Twitch VODs and Highlights.
* [`brotli`](https://github.com/google/brotli): Compresses Twitch VODs and Highlights chat files.
* [`chat_downloader`](https://github.com/xenova/chat-downloader): Downloads chat for YouTube videos.
* [`TwitchDownloaderCLI`](https://github.com/lay295/TwitchDownloader): Downloads chat for Twitch Clips.

## Build Dependencies
#### These programs need to be installed and in your path to compile this project.
* [`cargo`, `Rust 2021`](https://www.rust-lang.org/tools/install): Using `rustup` is recommended.
* [`pkg-config(1)`](https://www.freedesktop.org/wiki/Software/pkg-config/): Installing through your distro's official channels is recommended.
