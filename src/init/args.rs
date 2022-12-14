use super::{Info, VideoType, Videos};
use clap::CommandFactory;
use clap::{ArgGroup, Parser};
use derive_more::Constructor;
use std::{io::stdout, process::exit};

#[allow(clippy::struct_excessive_bools)]
#[derive(Parser)]
#[clap(
    author,
    version,
    about,
    override_usage = "archiver [OPTIONS] <TYPE> <INPUT>",
    arg_required_else_help = true
)]
#[clap(group(
  ArgGroup::new("type")
    .requires("input")
    .conflicts_with("generate")
    .args(&["vods", "highlights", "clips", "youtube"])
))]
#[clap(group(
  ArgGroup::new("input")
    .requires("type")
    .conflicts_with("generate")
    .args(&["channel", "videos"])
))]
struct Cli {
    /// Twitch VODs
    #[clap(long, help_heading = "TYPE")]
    vods: bool,

    /// Twitch Highlights
    #[clap(long, help_heading = "TYPE")]
    highlights: bool,

    /// Twitch Clips
    #[clap(long, help_heading = "TYPE")]
    clips: bool,

    /// YouTube
    #[clap(long, help_heading = "TYPE")]
    youtube: bool,

    /// Target channel (YouTube or Twitch)
    #[clap(long, short, help_heading = "INPUT")]
    channel: Option<String>,

    /// Target video (YouTube or Twitch)
    #[clap(help_heading = "INPUT")]
    videos: Option<String>,

    /// Number of video pieces to download simultaneously
    #[clap(long, short = 'N', default_value = "1")]
    threads: u16,

    /// How long ago to start searching for clips, refer to docs for format
    #[clap(
        short,
        long,
        default_value = "1week",
        help_heading = "CLIPS OPTIONS",
        value_name = "DURATION"
    )]
    range: String,

    /// Time interval to search for clips, shorter intervals will result in more requests but better results
    #[clap(
        short,
        long,
        default_value = "1hour",
        help_heading = "CLIPS OPTIONS",
        value_name = "DURATION"
    )]
    interval: String,

    /// Enable logging of external commands into files
    #[clap(short, long, takes_value = false)]
    logging: bool,

    /// Increase output verbosity
    #[clap(short, long, action = clap::ArgAction::Count, group = "output")]
    verbose: u8,

    /// Hide output, use twice to hide errors as well
    #[clap(short, long, action = clap::ArgAction::Count, group = "output")]
    silent: u8,

    /// Generate shell completion script
    #[clap(short, long, arg_enum, value_name = "SHELL", exclusive = true)]
    generate: Option<clap_complete_command::Shell>,
}

#[derive(Debug, Constructor)]
pub(super) struct Args {
    pub(super) videos: Videos,
    pub(super) video_type: VideoType,
    pub(super) verbosity: i16,
    pub(super) logging: bool,
    pub(super) range: String,
    pub(super) interval: String,
    pub(super) threads: u16,
}

pub(super) fn parse() -> Args {
    let cli = Cli::parse();
    if let Some(shell) = cli.generate {
        shell.generate(&mut Cli::command(), &mut stdout());
        exit(0);
    }

    let video_type: VideoType = match (cli.vods, cli.highlights, cli.clips, cli.youtube) {
        (true, _, _, _) => VideoType::Vod,
        (_, true, _, _) => VideoType::Highlight,
        (_, _, true, _) => VideoType::Clip,
        (_, _, _, true) => VideoType::YouTube,
        _ => unreachable!(),
    };

    let videos: Videos = match (cli.channel, cli.videos) {
        (Some(channel), _) => Videos::Channel(Info {
            data: channel,
            platform: video_type.clone(),
        }),
        (_, Some(videos)) => Videos::Direct(Info {
            data: videos,
            platform: video_type.clone(),
        }),
        _ => unreachable!(),
    };
    let verbosity = i16::from(cli.verbose) - i16::from(cli.silent);
    Args::new(
        videos,
        video_type,
        verbosity,
        cli.logging,
        cli.range,
        cli.interval,
        cli.threads,
    )
}
