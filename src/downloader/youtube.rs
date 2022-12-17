use super::{twitch::Video, Context};
use crate::{
  downloader::common,
  init::external::External,
  utils::{colorize, message, sanitize, VideoInfo},
  Error,
};
use colored::Color;
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use std::{
  cmp::min,
  fmt::{self, Display, Formatter},
  fs::OpenOptions,
  path::Path,
  process::{Command, Stdio},
};

lazy_static! {
  static ref ID_REGEX: Regex = unsafe { Regex::new(r"^([0-9a-zA-Z_-]{11})$").unwrap_unchecked() };
  static ref URL_REGEX: Regex = unsafe {
    Regex::new(r"^(?:https?://)?(?:www\.)?(?:youtu\.be/|youtube\.com(?:/embed/|/v/|/watch))(?:(?:&|\?)[^&]+)*(?:(?:&|\?)v=)?([0-9a-zA-Z_-]{11})(?:(?:&|\?)[^&]+)*(?:#.*)?$").unwrap_unchecked()
  };
  static ref CHAN_REGEX: Regex = unsafe { Regex::new(r"^([^?/\& *+]+)$").unwrap_unchecked() };
  static ref CHAN_URL_REGEX: Regex = unsafe {
    Regex::new(r"^(?:https?://)?(?:www\.)?youtube.com/(?:channel/|c/|user/|)([^?/\& *+]+)")
      .unwrap_unchecked()
  };
}

pub(super) fn download<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
  common::download(
    info,
    context,
    save_json,
    common::get_thumbnail,
    get_chat,
    process_chat,
    get_video,
  )
}

pub(super) fn get_ids<T: VideoInfo>(data: &str, context: &mut Context) -> Result<Vec<T>, Error> {
  common::get_ids(data, "", context, &[&ID_REGEX, &URL_REGEX], id2info)
}

fn id2info<T: VideoInfo>(
  ids: Vec<String>,
  _type: &str,
  context: &mut Context,
) -> Result<Vec<T>, Error> {
  let ids = ids
    .iter()
    .filter_map(|id| {
      let url = format!(
        "https://www.youtube.com/oembed?format=json&url=http://www.youtube.com/watch?v={id}"
      );
      let status = get_status(&url, context);
      if let Ok(status) = status {
        if status == StatusCode::OK {
          return Some(id.to_string());
        }
      }
      None
    })
    .collect::<Vec<String>>();
  get_info(ids, context)
}

fn get_info<T: VideoInfo>(mut ids: Vec<String>, context: &mut Context) -> Result<Vec<T>, Error> {
  let max = 50;
  let mut info = Vec::new();
  while !ids.is_empty() {
    let mut query = String::new();
    let mut iter = ids.drain(0..min(ids.len(), max));
    query.push_str(&iter.next().unwrap());
    for id in iter {
      query.push_str(format!("%2C{id}").as_str());
    }
    message(format!("[get_info] Query {query}"), context, 3);
    let url = format!("https://youtube.googleapis.com/youtube/v3/videos?part=snippet%2CcontentDetails%2Cstatistics&id={query}&maxResults={max}&key={}", context.token);
    message(format!("[get_info] URL: {url}"), context, 3);
    if let Ok(response) = get(&url, context) {
      message(format!("[get_info] Response: {response}"), context, 3);
      match serde_json::from_str::<YtResponse<T>>(&response) {
        Ok(mut data) => info.append(&mut data.items),
        Err(err) => message(
          colorize(
            Some("get_info"),
            &format!("JSON Error: {err}"),
            Color::BrightRed,
          ),
          context,
          -1,
        ),
      }
    }
  }
  message("[get_info] Info found:".to_string(), context, 3);
  for inf in info.iter() {
    message(format!("  {inf}"), context, 3);
  }
  Ok(info)
}

pub(super) fn get_channel_ids<T: VideoInfo>(
  channel: &str,
  context: &mut Context,
) -> Result<Vec<T>, Error> {
  let max = 50;
  let channel = common::regex_helper(channel, context, &[&CHAN_REGEX, &CHAN_URL_REGEX])?;
  let user = get_channel(&channel, context)?;
  message(
    colorize(
      Some("get_channel_ids"),
      &format!("Found channel with id {}", user.id),
      Color::BrightGreen,
    ),
    context,
    2,
  );
  let playlist = user.content_details.related_playlists.uploads;
  let mut videos = Vec::new();
  let mut after = String::new();
  loop {
    let url = format!(
      "https://youtube.googleapis.com/youtube/v3/playlistItems?part=snippet&maxResults={max}&playlistId={playlist}&key={}&pageToken={after}", context.token
    );
    message(format!("[get_channel_ids] URL: {url}"), context, 3);
    let response = get(&url, context)?;
    message(
      format!("[get_channel_ids] Response: {response}"),
      context,
      3,
    );
    let data: YtResponse<PlaylistItem> = match serde_json::from_str(&response) {
      Ok(data) => data,
      Err(err) => {
        message(
          colorize(
            Some("get_channel_ids"),
            &format!("Could not deserialize: {err}"),
            Color::BrightRed,
          ),
          context,
          -1,
        );
        return Err(err.into());
      }
    };
    for video in data.items {
      videos.push(video.snippet.resource_id.video_id.to_string());
    }
    match data.next_page_token {
      Some(cursor) if cursor != after || after.is_empty() => after = cursor,
      _ => break,
    }
  }
  get_info(videos, context)
}

fn get_channel(channel: &str, context: &mut Context) -> Result<Channel, Error> {
  let url = format!(
    "https://youtube.googleapis.com/youtube/v3/channels?part=contentDetails&id={channel}&key={}",
    context.token
  );
  let response = get(&url, context)?;
  let yt_response = serde_json::from_str::<YtResponse<Channel>>(&response);
  if let Ok(response) = yt_response {
    return Ok(response.items[0].clone());
  }
  let url = format!("https://youtube.googleapis.com/youtube/v3/channels?part=contentDetails&forUsername={channel}&key={}", context.token);
  let response = get(&url, context)?;
  let yt_response = serde_json::from_str::<YtResponse<Channel>>(&response);
  if let Ok(response) = yt_response {
    return Ok(response.items[0].clone());
  }
  Err(Error::NoMatches)
}

fn save_json<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
  let info: Video = info.to_video();
  common::save_json(&info, context)
}

fn get_chat(id: &str, context: &mut Context) -> Result<(), Error> {
  let chat_string = format!("{id}.chat.json");
  let chat = Path::new(&chat_string);
  if chat.exists() {
    return Err(Error::AlreadyExists);
  }
  if context.missing.contains(&External::Cd) {
    return Err(Error::MissingProgram(External::Cd));
  }
  let (log, err_log) = match context.logging {
    true => {
      let log_string = format!("{id}.chat.log");
      let log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_string)
        .unwrap();
      let err_string = format!("{id}.chat.err.log");
      let err_log = OpenOptions::new()
        .append(true)
        .create(true)
        .open(err_string)
        .unwrap();
      (log.into(), err_log.into())
    }
    false => (Stdio::null(), Stdio::null()),
  };
  let status = Command::new(External::Cd.command())
    .args(&[
      &format!("https://www.youtube.com/watch?v={id}"),
      "--output",
      &chat_string,
    ])
    .stdout(log)
    .stderr(err_log)
    .status()?;
  if !status.success() {
    return Err(Error::CommandFailed(External::Cd));
  }
  Ok(())
}

fn process_chat(_chat: &str, _context: &mut Context) -> Result<(), Error> {
  Err(Error::Expected)
}

fn get_video<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
  let video_filename = filename(info.title().to_string(), info.id().to_string());
  let video = Path::new(&video_filename);
  let url = format!("https://youtube.com/watch?v={}", info.id());
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
  let status = Command::new(External::YtDlp.command())
    .args(&[
      "-N",
      &context.threads.to_string(),
      "--compat-options",
      "filename",
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

fn filename(title: String, id: String) -> String {
  let title = sanitize(title, false);
  let filename = format!("{title}-{id}.mp4");
  filename
}

fn get_status(url: &str, context: &mut Context) -> Result<StatusCode, Error> {
  Ok(context.client.get(url).send()?.status())
}

fn get(url: &str, context: &mut Context) -> Result<String, Error> {
  Ok(
    context
      .client
      .get(url)
      .header("Accept", "application/json")
      .send()?
      .text()?,
  )
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Channel {
  id: String,
  content_details: ContentDetails,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ContentDetails {
  related_playlists: RelatedPlaylists,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RelatedPlaylists {
  uploads: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct YtResponse<T> {
  items: Vec<T>,
  prev_page_token: Option<String>,
  next_page_token: Option<String>,
  page_info: PageInfo,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PageInfo {
  total_results: u64,
  results_per_page: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PlaylistItem {
  snippet: ItemSnippet,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ItemSnippet {
  resource_id: ResourceId,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ResourceId {
  video_id: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct YtVideo {
  pub(super) id: String,
  pub(super) snippet: YtSnippet,
  pub(super) content_details: YtContentDetails,
  pub(super) statistics: YtStatistics,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct YtSnippet {
  pub(super) published_at: String,
  pub(super) channel_id: String,
  pub(super) title: String,
  pub(super) description: String,
  thumbnails: YtThumbnails,
  pub(super) channel_title: String,
  #[serde(default = "default_lang")]
  pub(super) default_language: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct YtContentDetails {
  pub(super) duration: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(super) struct YtStatistics {
  pub(super) view_count: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct YtThumbnails {
  default: Option<YtThumbnail>,
  medium: Option<YtThumbnail>,
  high: Option<YtThumbnail>,
  standard: Option<YtThumbnail>,
  maxres: Option<YtThumbnail>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct YtThumbnail {
  url: String,
}

impl VideoInfo for YtVideo {
  fn id(&self) -> &str {
    &self.id
  }
  fn title(&self) -> &str {
    &self.snippet.title
  }
  fn thumbnail_url(&self) -> &str {
    match self.snippet.thumbnails.maxres {
      Some(ref thumbnail) => &thumbnail.url,
      None => match self.snippet.thumbnails.standard {
        Some(ref thumbnail) => &thumbnail.url,
        None => match self.snippet.thumbnails.high {
          Some(ref thumbnail) => &thumbnail.url,
          None => match self.snippet.thumbnails.medium {
            Some(ref thumbnail) => &thumbnail.url,
            None => match self.snippet.thumbnails.default {
              Some(ref thumbnail) => &thumbnail.url,
              None => "",
            },
          },
        },
      },
    }
  }
  fn to_video(&self) -> Video {
    self.clone().into()
  }
}

impl Display for YtVideo {
  fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
    write!(
      f,
      "[{}] ({}) {}",
      self.id, self.snippet.channel_title, self.snippet.title
    )
  }
}

fn default_lang() -> String {
  "en".to_string()
}
