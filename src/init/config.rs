use crate::{utils::Spinner, Error};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub(super) fn get(verbosity: i8, spinner: &mut Spinner) -> Result<Config, Error> {
  let project_dirs = match ProjectDirs::from("moe", "Assistant", "archiver") {
    Some(dirs) => dirs,
    None => {
      return Err(Error::Config(
        "Could not find project directory".to_string(),
      ))
    }
  };

  if !project_dirs.config_dir().exists() {
    create_dir(&project_dirs.config_dir().to_path_buf())?;
  }
  let config_path = project_dirs.config_dir().join("config.toml");

  let config = match read_config(&config_path) {
    Ok(config) => config,
    Err(Error::ConfigFileMissing) => {
      create_config(&config_path)?;
      return Err(Error::Config(format!(
        "Config file missing: {}",
        config_path.display()
      )));
    }
    Err(Error::Io) => {
      return Err(Error::Config(format!(
        "Could not open config file: {}",
        config_path.display()
      )));
    }
    Err(err) => {
      if verbosity >= 3 {
        spinner.stop();
        println!("[config] {err}");
        spinner.start();
      }
      return Err(Error::Config(format!(
        "Could not read config file: {}",
        config_path.display()
      )));
    }
  };
  Ok(config)
}

fn read_config(config_path: &PathBuf) -> Result<Config, Error> {
  if config_path.exists() {
    let config_string = std::fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_string)?;
    Ok(config)
  } else {
    Err(Error::ConfigFileMissing)
  }
}

fn write_config(config_path: &PathBuf, config: &Config) -> Result<(), Error> {
  let config_string = toml::to_string(config)?;
  std::fs::write(config_path, config_string)?;
  Ok(())
}

fn create_config(config_path: &PathBuf) -> Result<(), Error> {
  match write_config(config_path, &Config::default()) {
    Ok(()) => Ok(()),
    Err(_) => Err(Error::Config(format!(
      "Could not create config file: {}",
      config_path.display()
    ))),
  }
}

fn create_dir(dir_path: &PathBuf) -> Result<(), Error> {
  match std::fs::create_dir_all(dir_path) {
    Ok(()) => Ok(()),
    Err(_) => Err(Error::Config(format!(
      "Could not create config directory: {}",
      dir_path.display()
    ))),
  }
}

#[derive(Debug, Deserialize, Default, Serialize)]
pub(super) struct Config {
  #[serde(default)]
  pub(super) twitch_client_id: String,
  #[serde(default)]
  pub(super) twitch_secret: String,
  #[serde(default)]
  pub(super) youtube_key: String,
}
