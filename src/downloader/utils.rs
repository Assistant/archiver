use super::{twitch::Video, Context};
use crate::Error;
use colored::{Color, Colorize};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{
    fmt::{self, Debug, Display, Formatter},
    fs::{File, OpenOptions},
    io::Write,
    path::Path,
    process::Stdio,
};
use terminal_spinners::{SpinnerBuilder, SpinnerHandle, DOTS2};
lazy_static! {
    static ref SANITIZE: Regex = unsafe { Regex::new(r"[0-9]+(?::[0-9]+)+").unwrap_unchecked() };
}

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

pub(super) fn sanitize(data: String, restricted: bool) -> String {
    let mut data = data;
    data = SANITIZE
        .replace_all(&data, |r: &Captures| {
            r.get(0).map_or("", |x| x.as_str()).replace(':', "_")
        })
        .to_string();
    data = data
        .chars()
        .filter(|c| match c {
            '?' | '\0'..='\u{1f}' | '\u{7f}' => false,
            '"' if restricted => false,
            _ => true,
        })
        .map(|c| match c {
            '\\' | '/' | '|' | '*' | '<' | '>' => '_',
            '\n' if !restricted => ' ',
            '!' | '&' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '$' | ';' | '1' | '^' | '#'
            | ' '
                if restricted =>
            {
                '_'
            }
            '"' => '\'',
            _ if c.is_whitespace() && restricted => '_',
            'Â' | 'Ã' | 'Ä' | 'À' | 'Á' | 'Å' if restricted => 'A',
            'â' | 'ã' | 'ä' | 'à' | 'á' | 'å' if restricted => 'a',
            'Ç' if restricted => 'C',
            'ç' if restricted => 'c',
            'È' | 'É' | 'Ê' | 'Ë' if restricted => 'E',
            'è' | 'é' | 'ê' | 'ë' if restricted => 'e',
            'Ì' | 'Í' | 'Î' | 'Ï' if restricted => 'I',
            'ì' | 'í' | 'î' | 'ï' if restricted => 'i',
            'Ð' if restricted => 'D',
            'ð' if restricted => 'd',
            'Ñ' if restricted => 'N',
            'ñ' if restricted => 'n',
            'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' if restricted => 'O',
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' if restricted => 'o',
            'Ù' | 'Ú' | 'Û' | 'Ü' if restricted => 'U',
            'ù' | 'ú' | 'û' | 'ü' if restricted => 'u',
            'Ý' if restricted => 'Y',
            'ý' if restricted => 'y',
            _ if restricted && c > '\u{7f}' => '_',
            _ => c,
        })
        .collect::<String>();
    if restricted {
        data = data.replace('Æ', "AE");
        data = data.replace('æ', "ae");
        data = data.replace('Œ', "OE");
        data = data.replace('œ', "oe");
        data = data.replace('Þ', "TH");
        data = data.replace('þ', "th");
        data = data.replace('ß', "ss");
        data = data.replace(':', "_-");
    } else {
        data = data.replace(':', " -");
    }
    while data.contains("__") {
        data = data.replace("__", "_");
    }
    if let Some(result) = data.strip_prefix('_') {
        data = result.to_string();
    }
    if let Some(result) = data.strip_suffix('_') {
        data = result.to_string();
    }
    if restricted && data.starts_with("-_") {
        data = data[2..].to_string();
    }
    if data.starts_with('-') {
        data = format!("_{}", &data[1..]);
    }
    if let Some(result) = data.strip_prefix('.') {
        data = result.to_string();
    }
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
    pub(crate) fn new(verbosity: i16) -> Self {
        Self {
            handle: None,
            message: None,
            verbosity,
        }
    }
    pub(crate) fn create(&mut self, message: &str) {
        if self.verbosity >= -1 {
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
