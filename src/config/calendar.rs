use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Config;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct AccountConfig {
    pub server: String,
    pub username: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CalendarConfig {
    pub accounts: Vec<AccountConfig>,
    #[serde(default = "CalendarConfig::default_database_path")]
    pub database_path: PathBuf,
}

impl Default for CalendarConfig {
    fn default() -> Self {
        Self {
            accounts: Vec::new(),
            database_path: Self::default_database_path(),
        }
    }
}

impl CalendarConfig {
    fn default_database_path() -> PathBuf {
        Config::base_dir().place_data_file("calendar.db").unwrap()
    }
}
