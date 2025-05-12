use self::external::External;
use crate::{utils::Spinner, Error};
use chrono::Duration;
use derive_more::Constructor;
use fancy_regex::Regex;
use reqwest::blocking::Client;
use std::sync::LazyLock;
mod args;
mod cli;
mod config;
pub(super) mod external;
mod token;

static SPLIT: LazyLock<Regex> =
    LazyLock::new(|| unsafe { Regex::new(r"([0-9]+[a-zA-Z]+)").unwrap_unchecked() });
static PAIR: LazyLock<Regex> =
    LazyLock::new(|| unsafe { Regex::new(r"([0-9]+)([a-zA-Z]+)").unwrap_unchecked() });
static ZERO: LazyLock<Duration> = LazyLock::new(Duration::zero);

#[derive(Debug, Constructor)]
pub(super) struct Input {
    pub(super) videos: Videos,
    pub(super) context: Context,
}

pub(super) fn run() -> Result<Input, Error> {
    let args = args::parse();
    let mut spinner = Spinner::new(args.verbosity, args.hide_spinners);

    spinner.create(" Checking external programs");
    let missing = external::find_missing(&args.video_type);
    spinner.end();
    if args.verbosity >= -1 {
        for command in &missing {
            command.missing();
        }
    };

    spinner.create(" Getting config");
    let config = match config::get(args.verbosity, &mut spinner) {
        Ok(config) => config,
        Err(error) => {
            spinner.end();
            return Err(error);
        }
    };
    spinner.end();

    spinner.create(" Checking tokens");
    let token_package = match token::get(&args.video_type, &config) {
        Ok(token_package) => token_package,
        Err(error) => {
            spinner.end();
            return Err(error);
        }
    };
    spinner.end();

    let (range, interval) = match args.video_type {
        VideoType::Clip => {
            spinner.create(" Parsing arguments");
            let range = parse_duration(&args.range);
            let interval = parse_duration(&args.interval);
            spinner.end();
            (range, interval)
        }
        _ => (*ZERO, *ZERO),
    };

    let context = Context {
        verbosity: args.verbosity,
        token: token_package.token,
        client: token_package.client,
        client_id: token_package.client_id,
        downloader: args.video_type,
        skip_video: args.skip_video,
        threads: args.threads,
        missing,
        range,
        interval,
        logging: args.logging,
        spinner,
        post_json: args.post_json,
        post_thumbnail: args.post_thumbnail,
        post_chat: args.post_chat,
        post_chat_process: args.post_chat_process,
        post_video: args.post_video,
    };
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
    pub(super) verbosity: i16,
    pub(super) token: String,
    pub(super) client: Client,
    pub(super) client_id: String,
    pub(super) downloader: VideoType,
    pub(super) skip_video: bool,
    pub(super) threads: u16,
    pub(super) missing: Vec<External>,
    pub(super) range: Duration,
    pub(super) interval: Duration,
    pub(super) logging: bool,
    pub(super) spinner: Spinner,
    pub(super) post_json: Option<String>,
    pub(super) post_thumbnail: Option<String>,
    pub(super) post_chat: Option<String>,
    pub(super) post_chat_process: Option<String>,
    pub(super) post_video: Option<String>,
}

fn parse_duration(text: &str) -> Duration {
    let mut duration = Duration::seconds(0);
    let result = SPLIT.captures_iter(text);
    let pairs = result
        .map(|c| c.unwrap().get(0).unwrap().as_str())
        .map(Into::into)
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
        let captures = PAIR.captures(text).unwrap().unwrap();
        let number = captures.get(1).unwrap().as_str().to_string();
        let unit = captures.get(2).unwrap().as_str().to_string();
        Time { number, unit }
    }
}
