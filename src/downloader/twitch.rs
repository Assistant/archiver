use super::common::{self, filename};
use super::utils::{colorize, error, loggers, message, VideoInfo};
use super::youtube::YtVideo;
use super::Context;
use crate::init::external::External;
use crate::Error;
use colored::Color;
use derive_more::Constructor;
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::fmt::{self, Debug, Display, Formatter};
use std::path::Path;
use std::process::Command;
use std::sync::LazyLock;

static RE: LazyLock<Regex> =
    LazyLock::new(|| unsafe { Regex::new(r"^[0-9]+$").unwrap_unchecked() });
static CHANNEL: LazyLock<Regex> = LazyLock::new(|| unsafe {
    Regex::new(r"^(?:https?://)?(?:www.)?(?:twitch.tv/)?(?:[^/]+/)?([^?/\& *+]+)")
        .unwrap_unchecked()
});
const PROCESSING: &str = "https://vod-secure.twitch.tv/_404/404_processing_%{width}x%{height}.png";

pub(super) fn id2info<T: VideoInfo>(
    ids: &[String],
    r#type: &str,
    context: &mut Context,
) -> Result<Vec<T>, Error> {
    let max = 100;
    message(&format!("[id2info] Getting info for {ids:?}"), context, 3);
    let mut info = Vec::new();
    let mut ids: Vec<String> = ids.to_vec();
    while !ids.is_empty() {
        let mut query = String::new();
        let mut iter = ids.drain(0..min(ids.len(), max));
        query.push_str(format!("?id={}", &iter.next().unwrap()).as_str());
        for id in iter {
            query.push_str(format!("&id={id}").as_str());
        }
        message(&format!("[id2info] Query: {query}"), context, 3);
        let endpoint = match r#type {
            "highlight" | "archive" => "videos",
            "clip" => "clips",
            _ => return Err(Error::NoType),
        };
        let url = format!("https://api.twitch.tv/helix/{endpoint}/{query}");
        message(&format!("[id2info] URL: {url}"), context, 3);
        if let Ok(response) = get(&url, context) {
            message(&format!("[id2info] Response: {response}"), context, 3);
            if let Ok(mut data) = serde_json::from_str::<TwitchResponse<T>>(&response) {
                info.append(&mut data.data);
            } else {
                message(
                    &colorize(Some("id2info"), "Could not deserialize.", Color::BrightRed),
                    context,
                    -1,
                );
            }
        }
    }
    message("[id2info] Info found:", context, 3);
    for inf in &info {
        message(&format!("  {inf}"), context, 3);
    }
    Ok(info)
}

pub(super) fn get_channel_ids<T: VideoInfo>(
    channel: &str,
    r#type: &str,
    context: &mut Context,
) -> Result<Vec<T>, Error> {
    let ChannelData { username, id } = get_channel(channel, context)?;
    message(
        &colorize(
            Some("get_channel_ids"),
            &format!("Found channel {username} with id {id}"),
            Color::BrightGreen,
        ),
        context,
        2,
    );
    let mut videos = Vec::new();
    let mut after = String::new();
    loop {
        let url = format!(
            "https://api.twitch.tv/helix/videos?user_id={id}&type={type}&first=100&after={after}"
        );
        message(&format!("[get_channel_ids] URL: {url}"), context, 3);
        let response = get(&url, context)?;
        message(
            &format!("[get_channel_ids] Response: {response}"),
            context,
            3,
        );
        let data: PagedTwitchResponse<T> = match serde_json::from_str(&response) {
            Ok(data) => data,
            Err(err) => {
                message(&format!("[get_channel_ids] JSON Error: {err}"), context, 3);
                return Err(err.into());
            }
        };
        for video in data.data {
            if video.thumbnail_url() != PROCESSING {
                videos.push(video);
            }
        }
        match data.pagination.cursor {
            Some(cursor) if cursor != after || after.is_empty() => after = cursor,
            _ => break,
        }
    }
    Ok(videos)
}

pub(super) fn get_chat(id: &str, context: &mut Context) -> Result<(), Error> {
    let chat_string = format!("{id}.ssa");
    let chat = Path::new(&chat_string);
    let json_string = format!("{id}.chat.json");
    let json = Path::new(&json_string);
    if chat.exists() && json.exists() {
        return Err(Error::AlreadyExists);
    }

    let mut ssa_status = true;
    let mut json_status = true;
    let missing_tcd = context.missing.contains(&External::Tcd);
    let missing_tdcli = context.missing.contains(&External::TdCli);

    if !missing_tcd && !chat.exists() {
        let (log, err_log) = loggers(&format!("{id}.chat"), context.logging);
        ssa_status = Command::new(External::Tcd.command())
            .args([
                "-f",
                "ssa",
                "-v",
                id,
                "--filename-format",
                "./{video_id}.{format}",
            ])
            .stdout(log)
            .stderr(err_log)
            .status()?
            .success();
    }

    if !missing_tdcli && !json.exists() {
        let (log, err_log) = loggers(&format!("{id}.json.chat"), context.logging);
        json_status = Command::new(External::TdCli.command())
            .args(["chatdownload", "-u", id, "-o", &format!("{id}.chat.json")])
            .stdout(log)
            .stderr(err_log)
            .status()?
            .success();
    }

    if missing_tcd {
        return Err(Error::MissingProgram(External::Tcd));
    }
    if !ssa_status {
        return Err(Error::CommandFailed(External::Tcd));
    }
    if missing_tdcli {
        return Err(Error::MissingProgram(External::TdCli));
    }
    if !json_status {
        return Err(Error::CommandFailed(External::TdCli));
    }
    Ok(())
}

pub(super) fn process_chat(id: &str, context: &mut Context) -> Result<(), Error> {
    let chat_string = format!("{id}.ssa");
    let chat = Path::new(&chat_string);
    let compressed_string = format!("{id}.ssa.br");
    let compressed = Path::new(&compressed_string);
    if compressed.exists() {
        return Err(Error::ProcessedChatAlreadyExists);
    }
    if context.missing.contains(&External::Brotli) {
        return Err(Error::MissingProgram(External::Brotli));
    }
    if !chat.exists() {
        return Err(Error::NoChatFound);
    }
    let (log, err_log) = loggers(&format!("{id}.process_chat"), context.logging);
    let status = Command::new(External::Brotli.command())
        .args(["-q", "11", format!("{id}.ssa").as_str()])
        .stdout(log)
        .stderr(err_log)
        .status()?;
    if !status.success() {
        return Err(Error::CommandFailed(External::Brotli));
    }
    Ok(())
}

pub(super) fn get_video<T: VideoInfo>(info: &T, context: &mut Context) -> Result<(), Error> {
    let video_filename = filename(info.title().to_string(), info.id());
    let video = Path::new(&video_filename);
    let url = format!("https://www.twitch.tv/videos/{}", info.id());
    if video.exists() {
        return Err(Error::AlreadyExists);
    }
    if context.missing.contains(&External::YtDlp) {
        return Err(Error::MissingProgram(External::YtDlp));
    }
    let (log, err_log) = loggers(&format!("{}.video", info.id()), context.logging);
    let status = Command::new(External::YtDlp.command())
        .args([
            "-N",
            &context.threads.to_string(),
            "--compat-options",
            "filename",
            "--downloader",
            "m3u8:ffmpeg",
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

#[derive(Debug, Constructor)]
pub(super) struct ChannelData {
    pub(super) username: String,
    pub(super) id: String,
}

#[derive(Debug, Deserialize)]
struct TwitchResponse<T> {
    data: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub(super) struct PagedTwitchResponse<T> {
    pub(super) data: Vec<T>,
    pub(super) pagination: Pagination,
}

#[derive(Debug, Deserialize)]
pub(super) struct Pagination {
    #[serde(default)]
    pub(super) cursor: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct Video {
    pub(super) id: String,
    pub(super) stream_id: Option<String>,
    pub(super) user_id: String,
    pub(super) user_login: String,
    pub(super) user_name: String,
    pub(super) title: String,
    pub(super) description: String,
    pub(super) created_at: String,
    pub(super) published_at: String,
    pub(super) url: String,
    pub(super) thumbnail_url: String,
    pub(super) viewable: String,
    pub(super) view_count: u64,
    pub(super) language: String,
    pub(super) r#type: String,
    pub(super) duration: String,
    #[serde(default)]
    pub(super) muted_segments: Option<Vec<MutedSegments>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub(super) struct MutedSegments {
    duration: u64,
    offset: u64,
}

impl VideoInfo for Video {
    fn id(&self) -> &str {
        &self.id
    }
    fn title(&self) -> &str {
        &self.title
    }
    fn thumbnail_url(&self) -> &str {
        &self.thumbnail_url
    }
    fn to_video(&self) -> Video {
        self.clone()
    }
}

impl Display for Video {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        write!(f, "[{}] ({}) {}", self.id, self.user_name, self.title)
    }
}

impl From<YtVideo> for Video {
    fn from(video: YtVideo) -> Self {
        Video {
            id: video.id.to_string(),
            stream_id: Some(video.id.to_string()),
            user_id: video.snippet.channel_id.to_string(),
            user_login: video.snippet.channel_id.to_string(),
            user_name: video.snippet.channel_title.to_string(),
            title: video.snippet.title.to_string(),
            description: video.snippet.description.to_string(),
            created_at: video.snippet.published_at.to_string(),
            published_at: video.snippet.published_at.to_string(),
            url: format!("https://www.youtube.com/watch?v={}", video.id),
            thumbnail_url: video.thumbnail_url().to_string(),
            viewable: "true".to_string(),
            view_count: video.statistics.view_count.parse().unwrap_or(0),
            language: video.snippet.default_language.to_string(),
            r#type: "youtube".to_string(),
            duration: video.content_details.duration.to_string(),
            muted_segments: None,
        }
    }
}

#[derive(Debug, Deserialize)]
struct User {
    id: String,
    login: String,
}

fn channel_regex(channel: &str) -> bool {
    RE.is_match(channel).unwrap_or(false)
}

fn get_channel_request(identifier: &str, context: &mut Context) -> Result<ChannelData, Error> {
    let url = format!("https://api.twitch.tv/helix/users?{identifier}");
    let response = get(&url, context)?;
    message(&format!("[channel_type] Response: {response}"), context, 3);
    let user_response: TwitchResponse<User> = serde_json::from_str(&response)?;
    message(
        &format!("[channel_type] user_response: {user_response:#?}"),
        context,
        3,
    );
    match user_response.data.len() {
        0 => Err(Error::NoMatches),
        1 => {
            let user_data = &user_response.data[0];
            let user = ChannelData::new(user_data.login.to_string(), user_data.id.to_string());
            message(&format!("[channel_type] ID: {user:?}"), context, 3);
            Ok(user)
        }
        _ => {
            error("???", Some(&["What the actual heck?"]));
            Err(Error::NoMatches)
        }
    }
}

pub(super) fn get_channel(channel: &str, context: &mut Context) -> Result<ChannelData, Error> {
    let channel = common::regex_helper(channel, context, &[&CHANNEL])?;
    let id = format!("id={channel}");
    let login = format!("login={channel}");
    if channel_regex(&channel) {
        match get_channel_request(&id, context) {
            Ok(user) => Ok(user),
            Err(_) => get_channel_request(&login, context),
        }
    } else {
        get_channel_request(&login, context)
    }
}

pub(super) fn get(url: &str, context: &mut Context) -> Result<String, Error> {
    Ok((context.client)
        .get(url)
        .header("Client-ID", &context.client_id)
        .bearer_auth(&context.token)
        .send()?
        .text()?)
}
