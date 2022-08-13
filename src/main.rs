mod downloader;
mod init;
use downloader::clip::Clip;
pub(crate) use downloader::error::Error;
use downloader::twitch::Video;
pub(crate) use downloader::utils;
use downloader::youtube::YtVideo;
use init::{Input, VideoType};
use utils::error;

fn main() {
  match init::run() {
    Ok(Input { videos, context }) => match &context.downloader {
      VideoType::Vod | VideoType::Highlight => videos.download::<Video>(&context),
      VideoType::Clip => videos.download::<Clip>(&context),
      VideoType::YouTube => videos.download::<YtVideo>(&context),
    },
    Err(Error::Token(msg) | Error::Config(msg)) => error(&msg, None),
    Err(_) => {}
  }
}
