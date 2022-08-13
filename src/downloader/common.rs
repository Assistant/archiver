use crate::{
  init::{Context, VideoType},
  utils::{
    download_file, error_msg, good_msg, message, regular_message, sanitize, split_videos,
    start_spinner, stop_spinner, warn_msg, write_file, VideoInfo,
  },
  Error,
};
use colored::Color;
use regex::Regex;
use std::path::Path;

pub(super) fn download<T: VideoInfo>(
  info: &T,
  context: &Context,
  save_json: fn(&T, &Context) -> Result<(), Error>,
  get_thumbnail: fn(&T, &Context) -> Result<(), Error>,
  get_chat: fn(&str, &Context) -> Result<(), Error>,
  process_chat: fn(&str, &Context) -> Result<(), Error>,
  get_video: fn(&T, &Context) -> Result<(), Error>,
) -> Result<(), Error> {
  let id = info.id();
  let chat_ext = match context.downloader {
    VideoType::Vod | VideoType::Highlight => ".ssa",
    VideoType::Clip | VideoType::YouTube => ".chat.json",
  };
  let video_title = match context.downloader {
    VideoType::Vod | VideoType::Highlight | VideoType::YouTube => {
      format!("{}-v{id}.mp4", sanitize(info.title().to_string(), false))
    }
    VideoType::Clip => format!("{id}.mp4"),
  };

  let spinner_text = format!(" Saving JSON {id}.json");
  let spinner = start_spinner(&spinner_text, context.verbosity);
  let result = save_json(info, context);
  stop_spinner(spinner);
  parse_result(&result, context, "json", "Download", &format!("{id}.json"));
  if let Err(error) = result {
    if error != Error::AlreadyExists {
      return Err(error);
    }
  }

  let spinner_text = format!(" Downloading {id}.jpg");
  let spinner = start_spinner(&spinner_text, context.verbosity);
  let result = get_thumbnail(info, context);
  stop_spinner(spinner);
  parse_result(
    &result,
    context,
    "thumbnail",
    "Download",
    &format!("{id}.jpg"),
  );

  let spinner_text = format!(" Downloading {id}{chat_ext}");
  let spinner = start_spinner(&spinner_text, context.verbosity);
  let result = get_chat(id, context);
  stop_spinner(spinner);
  parse_result(
    &result,
    context,
    "chat",
    "Download",
    &format!("{id}{chat_ext}"),
  );

  let spinner_text = format!(" Processing {id}{chat_ext}");
  let spinner = start_spinner(&spinner_text, context.verbosity);
  let result = process_chat(id, context);
  stop_spinner(spinner);
  parse_result(
    &result,
    context,
    "chat",
    "Process",
    &format!("{id}{chat_ext}.br"),
  );

  let spinner_text = format!(" Downloading {video_title}");
  let spinner = start_spinner(&spinner_text, context.verbosity);
  let result = get_video(info, context);
  stop_spinner(spinner);
  parse_result(&result, context, "video", "Download", &video_title);

  if context.verbosity >= 1 {
    message(
      None,
      &format!("Finished downloading {}", info.title()),
      Color::BrightGreen,
    );
  }
  Ok(())
}

fn parse_result(
  result: &Result<(), Error>,
  context: &Context,
  r#type: &str,
  verb: &str,
  filename: &str,
) {
  match result {
    Ok(()) => {
      good_msg(Some(r#type), format!("{verb}ed {filename}"), context);
    }
    Err(Error::AlreadyExists) => {
      warn_msg(Some(r#type), format!("Already exists: {filename}"), context);
    }
    Err(Error::Expected) => {}
    Err(_) => {
      error_msg(
        Some(r#type),
        format!("Failed to {} {filename}", verb.to_lowercase()),
        context,
      );
    }
  }
}

type Id2InfoHelper<T> = fn(Vec<String>, &str, &Context) -> Result<Vec<T>, Error>;

pub(super) fn save_json<T: VideoInfo>(info: &T, _context: &Context) -> Result<(), Error> {
  let path_string = format!("{}.json", &info.id());
  let path = Path::new(&path_string);
  if path.exists() {
    return Err(Error::AlreadyExists);
  }
  let mut json = serde_json::to_string_pretty(&info)?;
  json.push('\n');
  write_file(&path, json.as_bytes())
}

pub(super) fn get_thumbnail<T: VideoInfo>(info: &T, context: &Context) -> Result<(), Error> {
  let path_string = format!("{}.jpg", &info.id());
  let path = Path::new(&path_string);
  if path.exists() {
    return Err(Error::AlreadyExists);
  }
  let url = info.thumbnail_url().to_string();
  let url = url.replace("%{width}", "1920");
  let url = url.replace("%{height}", "1080");
  download_file(path, &url, context)
}

pub(super) fn get_ids<T: VideoInfo>(
  data: &str,
  r#type: &str,
  context: &Context,
  regexen: &[&'static Regex],
  id2info: Id2InfoHelper<T>,
) -> Result<Vec<T>, Error> {
  let label = "get_ids";
  if context.verbosity >= 3 {
    println!("[{label}] Getting ids for {data}");
  }
  let mut ids = Vec::new();
  for id in split_videos(data) {
    if let Ok(id) = regex_helper(&id, context, regexen) {
      ids.push(id);
    }
  }
  if context.verbosity >= 3 {
    println!("[{label}] ids found: {ids:?}"); // info
  }
  let info = id2info(ids, r#type, context)?;
  if info.is_empty() {
    Err(Error::NoMatches)
  } else {
    Ok(info)
  }
}

pub(super) fn regex_helper(
  text: &str,
  context: &Context,
  regexen: &[&Regex],
) -> Result<String, Error> {
  for regex in regexen.iter() {
    if let Some(captures) = regex.captures(text) {
      if let Some(capture) = captures.get(1) {
        if context.verbosity >= 3 {
          regular_message(Some("regex"), &format!("Captured {}", capture.as_str()));
        }
        return Ok(capture.as_str().to_string());
      }
    }
  }
  if context.verbosity >= 2 {
    message(
      Some("regex"),
      &format!("Could not match {text}"),
      Color::BrightRed,
    );
  }
  Err(Error::NoRegexMatch)
}

pub(super) fn filename(title: String, id: &str) -> String {
  let title = sanitize(title, false);
  let filename = format!("{title}-v{id}.mp4");
  filename
}
