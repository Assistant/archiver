use super::twitch::{get, ChannelData, PagedTwitchResponse};
use super::utils::{colorize, message, VideoInfo};
use super::{common, twitch, Context};
use crate::init::external::External;
use crate::Error;
use chrono::{SecondsFormat, Utc};
use colored::Color;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::fs::OpenOptions;
use std::path::Path;
use std::process::{Command, Stdio};

lazy_static! {
  static ref ID_REGEX: Regex = unsafe { Regex::new(r"^([A-z0-9-]+)$").unwrap_unchecked() };
  static ref URL_REGEX: Regex = unsafe {
    Regex::new(r"^(?:https://)?(?:clips\.|www\.)?twitch\.tv/([A-z0-9-]+)(?:\?.*)?$")
      .unwrap_unchecked()
  };
  static ref CHANNEL_REGEX: Regex = unsafe {
    Regex::new(r"^(?:https://)?(?:www\.)?twitch\.tv/(?:[^/]+)/clip/([A-z0-9-]+)(?:\?.*)?$")
      .unwrap_unchecked()
  };
}

pub(super) fn download<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
  common::download(
    info,
    context,
    common::save_json,
    common::get_thumbnail,
    get_chat,
    process_chat,
    get_video,
  )
}

pub(super) fn get_ids<T: VideoInfo>(data: &str, context: &mut Context) -> Result<Vec<T>, Error> {
  common::get_ids(
    data,
    "clip",
    context,
    &[&*ID_REGEX, &*URL_REGEX, &*CHANNEL_REGEX],
    twitch::id2info,
  )
}

pub(super) fn get_channel_ids<T: VideoInfo>(
  channel: &str,
  context: &mut Context,
) -> Result<Vec<T>, Error> {
  let ChannelData { username, id } = twitch::get_channel(channel, context)?;
  message(
    colorize(
      Some("get_channel_ids"),
      &format!("Found channel {username} with id {id}"),
      Color::BrightGreen,
    ),
    context,
    2,
  );
  let mut videos = Vec::new();
  let mut after = String::new();
  let interval = context.interval;
  let range = context.range;
  let now = Utc::now();
  let mut start = now - range;
  'date: loop {
    let end = start + interval;
    let start_string = start.to_rfc3339_opts(SecondsFormat::Secs, true);
    let end_string = end.to_rfc3339_opts(SecondsFormat::Secs, true);
    'page: loop {
      let url = format!(
        "https://api.twitch.tv/helix/clips?first=100&broadcaster_id={id}&after={after}&started_at={start_string}&ended_at={end_string}",
      );
      message(format!("[get_channel_ids] URL: {url}"), context, 3);
      let response = get(&url, context)?;
      message(
        format!("[get_channel_ids] Response: {response}"),
        context,
        3,
      );
      let data: PagedTwitchResponse<T> = match serde_json::from_str(&response) {
        Ok(data) => data,
        Err(err) => {
          message(format!("[get_channel_ids] JSON Error: {err}"), context, 2);
          break 'page;
        }
      };
      for video in data.data {
        videos.push(video);
      }
      match data.pagination.cursor {
        Some(cursor) if cursor != after || after.is_empty() => after = cursor,
        _ => break 'page,
      }
    }
    start += interval;
    if start > now {
      break 'date;
    }
  }
  Ok(videos)
}

// Chat for Twitch Clips is not working.
#[allow(unreachable_code, unused)]
fn get_chat(id: &str, context: &mut Context) -> Result<(), Error> {
  // Return an error since TwitchDownloaderCLI is not working currently
  return Err(Error::Expected);
  // todo!() Remove when TwitchDownloaderCLI is working, or an alternative is implemented
  let chat_string = format!("{id}.chat.json");
  let chat = Path::new(&chat_string);
  if chat.exists() {
    return Err(Error::AlreadyExists);
  }
  if context.missing.contains(&External::TdCli) {
    return Err(Error::MissingProgram(External::TdCli));
  }
  let (log, err_log) = match context.logging {
    true => {
      let log_string = format!("{}.chat.log", id);
      let log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_string)
        .unwrap();
      let err_string = format!("{}.chat.err.log", id);
      let err_log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(err_string)
        .unwrap();
      (log.into(), err_log.into())
    }
    false => (Stdio::null(), Stdio::null()),
  };
  let status = Command::new("TwitchDownloaderCLI")
    .args(&[
      "-m",
      "ChatDownload",
      "--id",
      id,
      "--embed-emotes",
      "-o",
      format!("{id}.chat.json").as_str(),
    ])
    .stdout(log)
    .stderr(err_log)
    .status()?;
  if !status.success() {
    return Err(Error::CommandFailed(External::TdCli));
  }
  Ok(())
}

fn process_chat(_id: &str, _context: &mut Context) -> Result<(), Error> {
  Err(Error::Expected)
}

fn get_video<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
  let video_filename = format!("{}.mp4", info.id());
  let video = Path::new(&video_filename);
  let url = format!("https://clips.twitch.tv/{}", info.id());
  if video.exists() {
    return Err(Error::AlreadyExists);
  }
  let (log, err_log) = match context.logging {
    true => {
      let log_string = format!("{}.video.log", info.id());
      let log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_string)
        .unwrap();
      let err_string = format!("{}.video.err.log", info.id());
      let err_log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(err_string)
        .unwrap();
      (log.into(), err_log.into())
    }
    false => (Stdio::null(), Stdio::null()),
  };
  if context.missing.contains(&External::YtDlp) {
    return Err(Error::MissingProgram(External::YtDlp));
  }
  let status = Command::new("yt-dlp")
    .args(&[
      "-N",
      &context.threads.to_string(),
      "-o",
      format!("{}.%(ext)s", info.id()).as_str(),
      &url,
    ])
    .stdout(log)
    .stderr(err_log)
    .status()?;
  if !status.success() {
    return Err(Error::CommandFailed(External::YtDlp));
  }
  Ok(())
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Clip {
  id: String,
  url: String,
  embed_url: String,
  broadcaster_id: String,
  broadcaster_name: String,
  creator_id: String,
  creator_name: String,
  video_id: String,
  game_id: String,
  language: String,
  title: String,
  view_count: u64,
  created_at: String,
  thumbnail_url: String,
  duration: f64,
  #[serde(default)]
  vod_offset: Option<u64>,
}

impl VideoInfo for Clip {
  fn id(&self) -> &str {
    &self.id
  }
  fn title(&self) -> &str {
    &self.title
  }
  fn thumbnail_url(&self) -> &str {
    &self.thumbnail_url
  }
  fn to_video(&self) -> twitch::Video {
    twitch::Video {
      id: self.id.clone(),
      stream_id: Some(self.video_id.clone()),
      user_id: self.broadcaster_id.clone(),
      user_login: self.broadcaster_name.clone(),
      user_name: self.broadcaster_name.clone(),
      title: self.title.clone(),
      description: "".to_string(),
      created_at: self.created_at.clone(),
      published_at: self.created_at.clone(),
      url: self.url.clone(),
      thumbnail_url: self.thumbnail_url.clone(),
      viewable: "lic".to_string(),
      view_count: self.view_count,
      language: self.language.clone(),
      r#type: "clip".to_string(),
      duration: self.duration.to_string(),
      muted_segments: None,
    }
  }
}

impl Display for Clip {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "[{}] ({}) {}",
      self.id, self.broadcaster_name, self.title
    )
  }
}
