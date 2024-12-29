use clap::{ArgGroup, Parser};

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
pub(crate) struct Cli {
    /// Twitch VODs
    #[clap(long, help_heading = "TYPE")]
    pub(crate) vods: bool,

    /// Twitch Highlights
    #[clap(long, help_heading = "TYPE")]
    pub(crate) highlights: bool,

    /// Twitch Clips
    #[clap(long, help_heading = "TYPE")]
    pub(crate) clips: bool,

    /// YouTube
    #[clap(long, help_heading = "TYPE")]
    pub(crate) youtube: bool,

    /// Target channel (YouTube or Twitch)
    #[clap(long, short, help_heading = "INPUT")]
    pub(crate) channel: Option<String>,

    /// Target video (YouTube or Twitch)
    #[clap(help_heading = "INPUT")]
    pub(crate) videos: Option<String>,

    /// Skip video download
    #[clap(long, short = 'K', takes_value = false)]
    pub(crate) skip_video: bool,

    /// Number of video pieces to download simultaneously
    #[clap(long, short = 'N', default_value = "1")]
    pub(crate) threads: u16,

    /// How long ago to start searching for clips, refer to docs for format
    #[clap(
        short,
        long,
        default_value = "1week",
        help_heading = "CLIPS OPTIONS",
        value_name = "DURATION"
    )]
    pub(crate) range: String,

    /// Time interval to search for clips, shorter intervals will result in more requests but better results
    #[clap(
        short,
        long,
        default_value = "1hour",
        help_heading = "CLIPS OPTIONS",
        value_name = "DURATION"
    )]
    pub(crate) interval: String,

    /// Enable logging of external commands into files
    #[clap(short, long, takes_value = false)]
    pub(crate) logging: bool,

    /// Increase output verbosity
    #[clap(short, long, action = clap::ArgAction::Count, group = "output")]
    pub(crate) verbose: u8,

    /// Hide output, use twice to hide errors as well
    #[clap(short, long, action = clap::ArgAction::Count, group = "output")]
    pub(crate) silent: u8,

    /// Hide spinners
    #[clap(short = 'q', long, takes_value = false)]
    pub(crate) hide_spinners: bool,
}
