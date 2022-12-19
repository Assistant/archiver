use super::External;
use std::fmt::Display;
use std::{fmt, io, string};

#[derive(Debug, PartialEq)]
pub(crate) enum Error {
    Io,
    JsonParseFailed,
    AlreadyExists,
    NoChatFound,
    ProcessedChatAlreadyExists,
    NoRegexMatch,
    NoMatches,
    NoType,
    ConfigFileMissing,
    ConfigParseFailed,
    ConfigSerializeFailed,
    Request,
    Format,
    MissingProgram(External),
    CommandFailed(External),
    Expected,
    Token(String),
    Config(String),
}
// todo!() Make better errors with information as to what went wrong

impl From<io::Error> for Error {
    fn from(_: io::Error) -> Error {
        Error::Io
    }
}

impl From<serde_json::Error> for Error {
    fn from(_: serde_json::Error) -> Error {
        Error::JsonParseFailed
    }
}

impl From<toml::de::Error> for Error {
    fn from(_: toml::de::Error) -> Error {
        Error::ConfigParseFailed
    }
}

impl From<toml::ser::Error> for Error {
    fn from(_: toml::ser::Error) -> Error {
        Error::ConfigSerializeFailed
    }
}

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Error {
        Error::Request
    }
}

impl From<fmt::Error> for Error {
    fn from(_: fmt::Error) -> Error {
        Error::Format
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(_: string::FromUtf8Error) -> Error {
        Error::Io
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::AlreadyExists => write!(f, "Already exists"),
            Error::NoChatFound => write!(f, "No chat found"),
            Error::ProcessedChatAlreadyExists => write!(f, "Compressed chat already exists"),
            Error::NoRegexMatch => write!(f, "No regex match"),
            Error::NoMatches => write!(f, "No matches"),
            Error::NoType => write!(f, "No type"),
            Error::Io => write!(f, "Reading JSON file failed"),
            Error::JsonParseFailed => write!(f, "Parsing JSON failed"),
            Error::ConfigFileMissing => write!(f, "Config file does not exist"),
            Error::ConfigParseFailed => write!(f, "Parsing config file failed"),
            Error::ConfigSerializeFailed => write!(f, "Serializing config file failed"),
            Error::Request => write!(f, "Request failed"),
            Error::Format => write!(f, "Formatting failed"),
            Error::MissingProgram(program) => write!(f, "Missing program: {program}"),
            Error::CommandFailed(program) => write!(f, "Command failed: {program}"),
            Error::Expected => write!(f, "This error is expected"),
            Error::Token(message) | Error::Config(message) => write!(f, "{message}"),
        }
    }
}
