use super::utils::VideoInfo;
use super::{common, twitch, Context};
use crate::Error;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
  static ref ID_REGEX: Regex = unsafe { Regex::new(r"^([0-9]+)$").unwrap_unchecked() };
  static ref URL_REGEX: Regex = unsafe {
    Regex::new(r"^(?:https://)?(?:www\.)?twitch\.tv/videos/([0-9]+)(?:\?.*)?$").unwrap_unchecked()
  };
}

pub(super) fn download<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
  common::download(
    info,
    context,
    common::save_json,
    common::get_thumbnail,
    twitch::get_chat,
    twitch::process_chat,
    twitch::get_video,
  )
}

pub(super) fn get_ids<T: VideoInfo>(data: &str, context: &mut Context) -> Result<Vec<T>, Error> {
  common::get_ids(
    data,
    "highlight",
    context,
    &[&*ID_REGEX, &*URL_REGEX],
    super::twitch::id2info,
  )
}

pub(super) fn get_channel_ids<T: VideoInfo>(
  channel: &str,
  context: &mut Context,
) -> Result<Vec<T>, Error> {
  twitch::get_channel_ids::<T>(channel, "highlight", context)
}
