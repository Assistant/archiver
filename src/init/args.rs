use super::{Info, VideoType, Videos};
use clap::Parser;
use derive_more::Constructor;

#[derive(Debug, Constructor)]
pub(super) struct Args {
    pub(super) videos: Videos,
    pub(super) video_type: VideoType,
    pub(super) verbosity: i16,
    pub(crate) hide_spinners: bool,
    pub(super) skip_video: bool,
    pub(super) logging: bool,
    pub(super) range: String,
    pub(super) interval: String,
    pub(super) threads: u16,
}

pub(super) fn parse() -> Args {
    let cli = crate::init::cli::Cli::parse();

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
        cli.hide_spinners,
        cli.skip_video,
        cli.logging,
        cli.range,
        cli.interval,
        cli.threads,
    )
}
