// use super::Context;
use super::VideoType;
use crate::utils;
// use crate::Error;
// use serde::Serialize;
// use serde_json::{ser::PrettyFormatter, Serializer, Value};
use std::fmt::Display;
use which::which;

// pub(super) fn init(context: &mut Context) {
//   match context.downloader {
//     VideoType::Vod | VideoType::Highlight => {
//       let _ = init_tcd(context);
//     }
//     _ => {}
//   }
// }

// fn init_tcd(context: &mut Context) -> Result<(), Error> {
//   if !context.missing.contains(&External::Tcd) {
//     let output = Command::new(External::Tcd.command()).arg("--settings").output()?;
//     let filename = String::from_utf8(output.stdout)?;
//     let filename = filename.trim();
//     let mut file = File::open(filename)?;
//     let mut settings = String::new();
//     file.read_to_string(&mut settings)?;
//     let mut config: Value = serde_json::from_str(&settings)?;
//     if config["client_id"].is_null() || config["client_secret"].is_null() {
//       config["client_id"] = serde_json::Value::String(context.client_id.clone());
//       config["client_secret"] = serde_json::Value::String(context.client_secret.clone());
//       let mut buf = Vec::new();
//       let formatter = PrettyFormatter::with_indent(b"    ");
//       let mut serializer = Serializer::with_formatter(&mut buf, formatter);
//       config.serialize(&mut serializer)?;
//       let mut file = File::create(filename)?;
//       file.write_all(&buf)?;
//     }
//   }
//   Ok(())
// }

pub(super) fn find_missing(video_type: &VideoType) -> Vec<External> {
    let commands = match video_type {
        VideoType::Vod | VideoType::Highlight | VideoType::Clip => {
            vec![External::TdCli, External::YtDlp]
        }
        VideoType::YouTube => vec![External::Cd, External::YtDlp],
    };
    commands.into_iter().filter(|c| !c.is_installed()).collect()
}

#[derive(Debug, PartialEq)]
pub(crate) enum External {
    // Tcd,
    YtDlp,
    Brotli,
    Cd,
    TdCli,
}

impl External {
    pub(crate) fn command(&self) -> &str {
        match self {
            // External::Tcd => "tcd",
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
            // External::Tcd => write!(
            //   f,
            //   "TwitchChatDownloader: https://github.com/PetterKraabol/Twitch-Chat-Downloader"
            // ),
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
