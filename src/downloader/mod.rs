pub(super) mod clip;
mod common;
pub(super) mod error;
mod highlight;
pub(super) mod twitch;
pub(super) mod utils;
mod vod;
pub(super) mod youtube;
use crate::init::{external::External, Context, VideoType, Videos};
use crate::Error;
use utils::VideoInfo;

use self::utils::{help_error, message};

impl Videos {
    pub(super) fn download<T: VideoInfo>(&self, context: &mut Context) {
        let (platform, ids): (&VideoType, Result<Vec<T>, Error>) = match self {
            Videos::Direct(info) => (
                &info.platform,
                info.platform.get_videos_ids(&info.data, context),
            ),
            Videos::Channel(info) => (
                &info.platform,
                info.platform.get_channel_ids(&info.data, context),
            ),
        };
        let ids = match ids {
            Ok(ids) => ids,
            Err(err) => {
                println!("{err}");
                return;
            } //todo!() Add error message
        };
        for info in ids {
            match platform.download(&info, context) {
                Ok(()) => {}
                Err(err) => {
                    // todo!() Maybe count errors?
                    message(&err.to_string(), context, -1);
                }
            }
        }
    }
}

impl VideoType {
    fn download<T: VideoInfo>(&self, info: &T, context: &mut Context) -> Result<(), Error> {
        match self {
            VideoType::Vod => vod::download(info, context),
            VideoType::Highlight => highlight::download(info, context),
            VideoType::Clip => clip::download(info, context),
            VideoType::YouTube => youtube::download(info, context),
        }
    }
    fn get_videos_ids<T: VideoInfo>(
        &self,
        data: &str,
        context: &mut Context,
    ) -> Result<Vec<T>, Error> {
        context.spinner.create(" Getting data from IDs");
        let info = match self {
            VideoType::Vod => vod::get_ids(data, context),
            VideoType::Highlight => highlight::get_ids(data, context),
            VideoType::Clip => clip::get_ids(data, context),
            VideoType::YouTube => youtube::get_ids(data, context),
        };
        context.spinner.end();
        match info {
            Ok(info) => Ok(info),
            Err(err) => {
                if context.verbosity >= -1 {
                    match err {
                        Error::NoMatches | Error::NoRegexMatch => {
                            help_error("No valid ids found in <INPUT>", None);
                        }
                        _ => {}
                    }
                }
                Err(err)
            }
        }
    }
    fn get_channel_ids<T: VideoInfo>(
        &self,
        channel: &str,
        context: &mut Context,
    ) -> Result<Vec<T>, Error> {
        context.spinner.create(" Getting channel data");
        let info = match self {
            VideoType::Vod => vod::get_channel_ids(channel, context),
            VideoType::Highlight => highlight::get_channel_ids(channel, context),
            VideoType::Clip => clip::get_channel_ids(channel, context),
            VideoType::YouTube => youtube::get_channel_ids(channel, context),
        };
        context.spinner.end();
        info
    }
}
