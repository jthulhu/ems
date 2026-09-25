use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use email::EmailConfig;
use emer::raise;
use serde::{Deserialize, Serialize};
use toml::to_string_pretty;

pub mod calendar;
pub mod email;

pub use calendar::CalendarConfig;
use xdg::BaseDirectories;

use crate::error::{ErrorKind, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub calendar: CalendarConfig,
    pub email: EmailConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            calendar: Default::default(),
            database_path: Self::default_database_path(),
        }
    }
}

impl Config {
    pub(self) fn base_dir() -> BaseDirectories {
        BaseDirectories::with_prefix("ems")
    }

    pub fn write_to_file() -> Result<PathBuf> {
        let base_dir = Self::base_dir();
        if let Some(path) = base_dir.find_config_file("config.toml") {
            Ok(path)
        } else {
            let path = base_dir
                .place_config_file("config.toml")
                .map_err(|error| raise!(ErrorKind::FileConfigCreation(error)))?;
            let default_config = Self::default();
            let mut file = File::create(&path).map_err(|error| {
                raise!(ErrorKind::CreateFile {
                    path: path.clone(),
                    error
                })
            })?;
            file.write_all(
                to_string_pretty(&default_config)
                    .map_err(|error| raise!(ErrorKind::TomlSer(error)))?
                    .as_bytes(),
            )
            .map_err(|error| {
                raise!(ErrorKind::WriteFile {
                    path: path.clone(),
                    error
                })
            })?;
            Ok(path)
        }
    }

    pub fn new() -> Result<Self> {
        let config_path = Self::write_to_file()?;
        let mut config_file = File::open(&config_path).map_err(|error| {
            raise!(ErrorKind::OpenFile {
                path: config_path.clone(),
                error
            })
        })?;
        let mut buffer = String::new();
        config_file.read_to_string(&mut buffer).map_err(|error| {
            raise!(ErrorKind::ReadFile {
                path: config_path.clone(),
                error
            })
        })?;
        let config =
            toml::from_str(&buffer).map_err(|error| raise!(ErrorKind::ParseConfig(error)))?;
        Ok(config)
    }
}
