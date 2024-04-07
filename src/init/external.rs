use super::VideoType;
use crate::utils;
use std::fmt::Display;
use which::which;

pub(super) fn find_missing(video_type: &VideoType) -> Vec<External> {
    let commands = match video_type {
        VideoType::Vod | VideoType::Highlight => vec![External::Tcd, External::YtDlp],
        VideoType::Clip => vec![External::TdCli, External::YtDlp],
        VideoType::YouTube => vec![External::Cd, External::YtDlp],
    };
    commands.into_iter().filter(|c| !c.is_installed()).collect()
}

#[derive(Debug, PartialEq)]
pub(crate) enum External {
    Tcd,
    YtDlp,
    Brotli,
    Cd,
    TdCli,
}

impl External {
    pub(crate) fn command(&self) -> &str {
        match self {
            External::Tcd => "tcd",
            External::YtDlp => "yt-dlp",
            External::Brotli => "brotli",
            External::Cd => "chat_downloader",
            External::TdCli => "TwitchDownloaderCLI",
        }
    }
    fn is_installed(&self) -> bool {
        which(self.command()).is_ok()
    }
    pub(super) fn missing(&self) {
        utils::error(format!("Missing external program: {self}").as_str(), None);
    }
}

impl Display for External {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            External::Tcd => write!(
                f,
                "TwitchChatDownloader: https://github.com/TheDrHax/Twitch-Chat-Downloader"
            ),
            External::YtDlp => write!(f, "yt-dlp: https://github.com/yt-dlp/yt-dlp"),
            External::Brotli => write!(f, "brotli: https://github.com/google/brotli"),
            External::Cd => write!(
                f,
                "chat_downloader: https://github.com/xenova/chat-downloader"
            ),
            External::TdCli => write!(
                f,
                "TwitchDownloaderCLI: https://github.com/lay295/TwitchDownloader"
            ),
        }
    }
}
