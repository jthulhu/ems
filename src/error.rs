use std::{io, path::PathBuf};

use sea_orm::DbErr;
use thiserror::Error;
use tokio::{sync::mpsc, task::JoinError};
use tuta_sdk::login::LoginError;

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("SQL error")]
    Sqlite(#[source] DbErr),
    #[error("wrong ical source {0}")]
    IcalParse(String),
    #[error("wrong ical recurrence")]
    IcalRecurrence(#[source] icalendar::RecurrenceError),
    #[error("event doesn't have a start timestamp")]
    IcalNoStart,
    #[error("poisoned mutex")]
    SyncMutexPoison,
    #[error("join error")]
    Join(#[source] JoinError),
    #[error("dav error")]
    Dav(#[source] fast_dav_rs::Error),
    #[error("keyring error")]
    Keyring(#[source] oo7::Error),
    #[error("couldn't create the config file")]
    FileConfigCreation(#[source] io::Error),
    #[error("couldn't create a file at `{}`", .path.display())]
    CreateFile {
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("couldn't write to file `{}`", .path.display())]
    WriteFile {
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("couldn't open file `{}`", .path.display())]
    OpenFile {
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("couldn't read file `{}`", .path.display())]
    ReadFile {
        path: PathBuf,
        #[source]
        error: io::Error,
    },
    #[error("couldn't parse the config file")]
    ParseConfig(#[source] toml::de::Error),
    #[error("couldn't serialize a TOML value")]
    TomlSer(#[source] toml::ser::Error),
    #[error("couldn't fetch an entry in the password store")]
    Pass(#[source] libpass::PassError),
    #[error("the password entry is a directory, not a file")]
    PassEntryDir,
    #[error("channel crashed, something bad happened")]
    Channel,
    #[error("tuta login error")]
    TutaLogin(#[source] LoginError),
}

pub type Error = emer::Error<ErrorKind>;
pub type Result<T> = emer::Result<T, ErrorKind>;
