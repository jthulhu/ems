use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::Config;

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct JmapAccountConfig {
    email: String,
    server: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub struct EmailConfig {
    pub jmap_accounts: Vec<JmapAccountConfig>,
    #[serde(default = "EmailConfig::default_database_path")]
    pub database_path: PathBuf,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            jmap_accounts: Vec::new(),
            database_path: Self::default_database_path(),
        }
    }
}

impl EmailConfig {
    fn default_database_path() -> PathBuf {
        Config::base_dir().place_data_file("email.db").unwrap()
    }
}
