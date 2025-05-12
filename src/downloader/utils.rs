use super::{twitch::Video, Context};
use crate::Error;
use colored::{Color, Colorize};
use fancy_regex::{Captures, Regex};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::HashMap;
use std::fmt::{self, Debug, Display, Formatter};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::LazyLock;
use strfmt::strfmt;
use terminal_spinners::{SpinnerBuilder, SpinnerHandle, DOTS2};
use unicode_general_category::{get_general_category, GeneralCategory};
use unicode_normalization::UnicodeNormalization;

static TIMESTAMP: LazyLock<Regex> =
    LazyLock::new(|| unsafe { Regex::new(r"[0-9]+(?::[0-9]+)+").unwrap_unchecked() });
static DEDUP: LazyLock<Regex> =
    LazyLock::new(|| unsafe { Regex::new(r"(\0.)(?:(?=\1)..)+").unwrap_unchecked() });
static STRIP: LazyLock<Regex> = LazyLock::new(|| unsafe {
    Regex::new(r"^\0.(?:\0.|[ _-])*|(?:\0.|[ _-])*$").unwrap_unchecked()
});
static ACCENTS: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    let source = "ÂÃÄÀÁÅÆÇÈÉÊËÌÍÎÏÐÑÒÓÔÕÖŐØŒÙÚÛÜŰÝÞßàáâãäåæçèéêëìíîïðñòóôõöőøœùúûüűýþÿ";
    let replacements = [
        "A", "A", "A", "A", "A", "A", "AE", "C", "E", "E", "E", "E", "I", "I", "I", "I", "D", "N",
        "O", "O", "O", "O", "O", "O", "O", "OE", "U", "U", "U", "U", "U", "Y", "TH", "ss", "a",
        "a", "a", "a", "a", "a", "ae", "c", "e", "e", "e", "e", "i", "i", "i", "i", "d", "n", "o",
        "o", "o", "o", "o", "o", "o", "oe", "u", "u", "u", "u", "u", "y", "th", "y",
    ];
    source.chars().zip(replacements).collect()
});
static SYMBOLS: LazyLock<HashMap<char, &str>> = LazyLock::new(|| {
    let replacements = ["＂", "＊", "：", "＜", "＞", "？", "｜", "⧸", "⧹"];
    "\"*:<>?|/\\".chars().zip(replacements).collect()
});

pub(crate) trait VideoInfo: Debug + Display + Serialize + DeserializeOwned {
    fn title(&self) -> &str;
    fn id(&self) -> &str;
    fn thumbnail_url(&self) -> &str;
    fn to_video(&self) -> Video;
}

pub(super) fn split_videos(data: &str) -> Vec<String> {
    data.split(',').map(|s| s.trim().to_string()).collect()
}

pub(super) fn colorize(label: Option<&str>, message: &str, color: Color) -> String {
    if let Some(label) = label {
        format!(
            "{}{label}{} {}",
            "[".bold().color(color),
            "]".bold().color(color),
            message.bold().color(color)
        )
    } else {
        message.bold().color(color).to_string()
    }
}

pub(super) fn message(msg: &str, context: &mut Context, threshold: i16) {
    if context.verbosity >= threshold {
        context.spinner.stop();
        println!("{msg}");
        context.spinner.start();
    }
}

pub(super) fn help_error(message: &str, extra: Option<&[&str]>) {
    error(message, extra);
    eprintln!("\nUSAGE:\n    archiver [OPTIONS] <TYPE> <INPUT>");
    eprintln!("\nFor more information try {}", "archiver --help".green());
}

pub(super) fn good_msg<T: Into<String>>(label: Option<&str>, msg: T, context: &mut Context) {
    message(
        &colorize(label, &msg.into(), Color::BrightGreen),
        context,
        0,
    );
}

pub(super) fn warn_msg<T: Into<String>>(label: Option<&str>, msg: T, context: &mut Context) {
    message(
        &colorize(label, &msg.into(), Color::BrightYellow),
        context,
        1,
    );
}

pub(super) fn error_msg<T: Into<String>>(label: Option<&str>, msg: T, context: &mut Context) {
    message(&colorize(label, &msg.into(), Color::BrightRed), context, -1);
}

pub(crate) fn error(message: &str, extra: Option<&[&str]>) {
    eprintln!("{} {message}", "error:".bold().bright_red());
    if let Some(extra) = extra {
        for line in extra {
            eprintln!("{line}");
        }
    }
}

pub(crate) fn loggers(name: &str, enabled: bool) -> (Stdio, Stdio) {
    if enabled {
        let log_string = format!("{name}.log");
        let log = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_string)
            .unwrap();
        let err_string = format!("{name}.err.log");
        let err_log = OpenOptions::new()
            .append(true)
            .create(true)
            .open(err_string)
            .unwrap();
        (log.into(), err_log.into())
    } else {
        (Stdio::null(), Stdio::null())
    }
}

pub(super) fn write_file(path: impl AsRef<Path>, bytes: &[u8]) -> Result<(), Error> {
    let path = path.as_ref();
    if path.exists() {
        return Err(Error::AlreadyExists);
    }
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    Ok(())
}

pub(super) fn download_file(
    path: impl AsRef<Path>,
    url: &str,
    context: &mut Context,
) -> Result<(), Error> {
    let response = context.client.get(url).send()?;
    let bytes = response.bytes()?;
    write_file(path, &bytes)
}

fn is_cm(c: char) -> bool {
    use GeneralCategory::{
        Control, EnclosingMark, Format, NonspacingMark, PrivateUse, SpacingMark, Surrogate,
    };
    matches!(
        get_general_category(c),
        Control | Format | Surrogate | PrivateUse | SpacingMark | EnclosingMark | NonspacingMark
    )
}

fn replace_insane(character: char, restrict: bool) -> String {
    let string = match character {
        c if restrict && ACCENTS.contains_key(&c) => ACCENTS.get(&c).unwrap(),
        c if !restrict && c == '\n' => "\0 ",
        c if !restrict && SYMBOLS.contains_key(&c) => SYMBOLS.get(&c).unwrap(),
        '?' | '\0'..='\u{1f}' | '\u{7f}' => "",
        '"' if restrict => "",
        '"' => "'",
        ':' if restrict => "\0_\0-",
        ':' => "\0 \0-",
        c if "\\/|*<>".contains(c) => "\0_",
        c if restrict && ("!&'()[]{}&;1^,#".contains(c) || c == ' ') => "\0_",
        c if restrict && c > '\u{7f}' && is_cm(c) => "",
        c if restrict && c > '\u{7f}' => "\0_",

        c => &c.to_string(),
    };
    string.to_owned()
}

pub(super) fn sanitize(data: String, restricted: bool) -> String {
    let mut data = if restricted {
        data.nfkc().to_string()
    } else {
        data
    };
    data = TIMESTAMP
        .replace_all(&data, |r: &Captures| {
            r.get(0).map_or("", |x| x.as_str()).replace(':', "_")
        })
        .to_string();
    data = data
        .chars()
        .map(|c| replace_insane(c, restricted))
        .collect();

    data = DEDUP.replace_all(&data, "$1").to_string();
    data = STRIP.replace_all(&data, "").to_string();

    data = data.replace('\0', "");

    while data.contains("__") {
        data = data.replace("__", "_");
    }
    data = data.trim_matches('_').to_string();
    if restricted && data.starts_with("-_") {
        data = data[2..].to_string();
    }
    if data.starts_with('-') {
        data = format!("_{}", &data[1..]);
    }
    data = data.trim_start_matches('.').to_string();
    if data.is_empty() {
        "_".to_string()
    } else {
        data
    }
}

pub(crate) struct Spinner {
    handle: Option<SpinnerHandle>,
    message: Option<String>,
    verbosity: i16,
    hidden: bool,
}

impl Debug for Spinner {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let handle = match &self.handle {
            Some(_) => "Some",
            None => "None",
        };
        write!(
            f,
            "Spinner {{ handle: {handle}, message: {:?} }}",
            self.message
        )
    }
}

impl Spinner {
    pub(crate) fn new(verbosity: i16, hidden: bool) -> Self {
        Self {
            handle: None,
            message: None,
            verbosity,
            hidden,
        }
    }
    pub(crate) fn create(&mut self, message: &str) {
        if self.verbosity >= -1 && !self.hidden {
            self.message = Some(message.to_string());
            self.handle = Some(
                SpinnerBuilder::new()
                    .spinner(&DOTS2)
                    .text(message.to_string())
                    .start(),
            );
        }
    }
    pub(crate) fn start(&mut self) {
        if self.verbosity >= -1 {
            if let Some(msg) = self.message.clone() {
                self.create(&msg);
            }
        }
    }
    pub(crate) fn stop(&mut self) {
        if let Some(handle) = self.handle.take() {
            handle.stop_and_clear();
        }
    }
    pub(crate) fn end(&mut self) {
        self.stop();
        self.message = None;
    }
}

pub(crate) fn run_template(template: &str, vars: &HashMap<String, String>) -> Result<(), Error> {
    match Command::new("bash")
        .arg("-c")
        .arg(&strfmt(template, vars)?)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()?
        .success()
    {
        true => Ok(()),
        false => Err(Error::Template),
    }
}
