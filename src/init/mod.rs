use self::external::External;
use crate::utils::{start_spinner, stop_spinner};
use crate::Error;
use chrono::Duration;
use derive_more::Constructor;
use regex::Regex;
use reqwest::blocking::Client;
mod args;
use lazy_static::lazy_static;
mod config;
pub(super) mod external;
mod token;

lazy_static! {
  static ref SPLIT: Regex = unsafe { Regex::new(r"([0-9]+[a-zA-Z]+)").unwrap_unchecked() };
  static ref PAIR: Regex = unsafe { Regex::new(r"([0-9]+)([a-zA-Z]+)").unwrap_unchecked() };
  static ref ZERO: Duration = Duration::zero();
}

#[derive(Debug, Constructor)]
pub(super) struct Input {
  pub(super) videos: Videos,
  pub(super) context: Context,
}

pub(super) fn run() -> Result<Input, Error> {
  let args = args::parse()?;

  let spinner = start_spinner(" Checking external programs", args.verbosity);
  let missing = external::find_missing(&args.video_type);
  stop_spinner(spinner);
  if args.verbosity >= -1 {
    for command in &missing {
      command.missing();
    }
  };

  let spinner = start_spinner(" Getting config", args.verbosity);
  let config = match config::get(args.verbosity) {
    Ok(config) => config,
    Err(error) => {
      stop_spinner(spinner);
      return Err(error);
    }
  };
  stop_spinner(spinner);

  let spinner = start_spinner(" Checking tokens", args.verbosity);
  let token_package = match token::get(&args.video_type, &config) {
    Ok(token_package) => token_package,
    Err(error) => {
      stop_spinner(spinner);
      return Err(error);
    }
  };
  stop_spinner(spinner);

  let (range, interval) = match args.video_type {
    VideoType::Clip => {
      let spinner = start_spinner(" Parsing arguments", args.verbosity);
      let range = parse_duration(&args.range);
      let interval = parse_duration(&args.interval);
      stop_spinner(spinner);
      (range, interval)
    }
    _ => (*ZERO, *ZERO),
  };

  let context = Context {
    verbosity: args.verbosity,
    token: token_package.token,
    client: token_package.client,
    client_id: token_package.client_id,
    client_secret: token_package.client_secret,
    downloader: args.video_type,
    threads: args.threads,
    missing,
    range,
    interval,
    logging: args.logging,
  };
  external::init(&context);
  Ok(Input::new(args.videos, context))
}

#[derive(Debug)]
pub(super) enum Videos {
  Direct(Info),
  Channel(Info),
}

#[derive(Debug)]
pub(super) struct Info {
  pub(super) data: String,
  pub(super) platform: VideoType,
}

#[derive(Debug, Clone)]
pub(super) enum VideoType {
  Vod,
  Highlight,
  Clip,
  YouTube,
}

#[derive(Debug)]
pub(super) struct Context {
  pub(super) verbosity: i8,
  pub(super) token: String,
  pub(super) client: Client,
  pub(super) client_id: String,
  client_secret: String,
  pub(super) downloader: VideoType,
  pub(super) threads: u16,
  pub(super) missing: Vec<External>,
  pub(super) range: Duration,
  pub(super) interval: Duration,
  pub(super) logging: bool,
}

fn parse_duration(text: &str) -> Duration {
  let mut duration = Duration::seconds(0);
  let result = SPLIT.captures_iter(text);
  let pairs = result
    .map(|c| c.get(0).unwrap().as_str())
    .map(|m| m.into())
    .collect::<Vec<Time>>();
  for pair in pairs {
    let Time { number, unit } = pair;
    let number = number.parse::<i64>().unwrap();
    match unit.to_lowercase().as_str() {
      "s" | "second" | "seconds" => add(&mut duration, Duration::seconds(number)),
      "m" | "minute" | "minutes" => add(&mut duration, Duration::minutes(number)),
      "h" | "hour" | "hours" => add(&mut duration, Duration::hours(number)),
      "d" | "day" | "days" => add(&mut duration, Duration::days(number)),
      "w" | "week" | "weeks" => add(&mut duration, Duration::weeks(number)),
      _ => {}
    }
  }
  duration
}

fn add(a: &mut Duration, b: Duration) {
  *a = *a + b;
}

#[derive(Debug)]
struct Time {
  number: String,
  unit: String,
}

impl From<&str> for Time {
  fn from(text: &str) -> Self {
    let captures = PAIR.captures(text).unwrap();
    let number = captures.get(1).unwrap().as_str().to_string();
    let unit = captures.get(2).unwrap().as_str().to_string();
    Time { number, unit }
  }
}
