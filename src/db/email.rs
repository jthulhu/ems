use std::{
    collections::HashSet,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use emer::{raise, throw};
use libpass::StoreEntry;
use migrations::TutaMigrationHandler;
use sea_orm::{
    ActiveValue, ColumnTrait, ConnectOptions, ConnectionTrait, Database, DatabaseConnection,
    EntityTrait, QueryFilter, TransactionTrait,
};
use tuta_account::Entity;
use tuta_sdk::{
    GeneratedId, HeadersProvider, LoggedInSdk, Sdk,
    bindings::{
        file_client::{FileClient, FileClientError},
        rest_client::RestClient,
    },
    crypto_entity_client::CryptoEntityClient,
    login::Credentials,
    net::native_rest_client::NativeRestClient,
    services::service_executor::{self, ServiceExecutor},
};
use xdg::BaseDirectories;

use crate::{
    config::email::TutaAccountConfig,
    error::{ErrorKind, Result},
    uring::{self, DiskInterface},
};

use super::passphrase::get_passphrase;

mod email_account;
mod email_data;
mod email_folder;
mod migrations;
mod tuta_account;
mod tuta_credential;

pub struct TutaSession {
    pub session: LoggedInSdk,
    pub email: String,
    pub token: String,
}

impl TutaSession {
    async fn new(
        username: &str,
        password: &str,
        api_url: &str,
        base_dir: BaseDirectories,
        disk: DiskInterface,
    ) -> Result<Self> {
        let rest_client: Arc<dyn RestClient> = Arc::new(NativeRestClient::try_new().unwrap());
        let file_client: Arc<dyn FileClient> = Arc::new(FileMapper::new(username, base_dir, disk));
        let sdk = Sdk::new_without_suspension(api_url.to_string(), rest_client, file_client);
        // sdk.
        todo!()
    }

    fn crypto_client(&self) -> Arc<CryptoEntityClient> {
        self.session.mail_facade().get_crypto_entity_client()
    }
}

/// File mapper for the Tuta SDK.  Tuta abstracts over the filesystem, asking for a key/value
/// map.  Storing the values in a database is a bad idea, Tuta is already implementing an
/// encrypted database.
pub struct FileMapper {
    email: String,
    base_dir: BaseDirectories,
    disk: DiskInterface,
}

impl FileMapper {
    pub fn new(email: impl ToString, base_dir: BaseDirectories, disk: DiskInterface) -> Self {
        Self {
            email: email.to_string(),
            base_dir,
            disk,
        }
    }

    fn file_to_store(&self, key: String) -> std::result::Result<PathBuf, io::Error> {
        self.base_dir
            .place_data_file(format!("{}/{key}", self.email))
    }
}

#[async_trait]
impl FileClient for FileMapper {
    async fn persist_content(
        &self,
        key: String,
        content: Vec<u8>,
    ) -> std::result::Result<(), FileClientError> {
        let content_path = self.file_to_store(key).map_err(|error| error.kind())?;
        self.disk
            .write(content_path, content)
            .await
            .map_err(|error| match error {
                uring::Error::Channel => FileClientError::Unknown,
                uring::Error::Io(error) => error.kind().into(),
            })?;
        Ok(())
    }

    async fn read_content(&self, key: String) -> std::result::Result<Vec<u8>, FileClientError> {
        let content_path = self.file_to_store(key).map_err(|error| error.kind())?;
        let data = self
            .disk
            .read(content_path)
            .await
            .map_err(|error| match error {
                uring::Error::Channel => FileClientError::Unknown,
                uring::Error::Io(error) => error.kind().into(),
            })?;
        Ok(data)
    }
}

#[derive(Debug, Clone)]
pub struct EmailStorage {
    tuta_db: DatabaseConnection,
    email_db: DatabaseConnection,
}

pub struct TutaAccount {}

impl EmailStorage {
    pub async fn open(
        tuta_path: impl AsRef<Path>,
        email_path: impl AsRef<Path>,
        app_name: &str,
    ) -> Result<Self> {
        let tuta_db = {
            let tuta_passphrase = get_passphrase(app_name, "tuta").await?;
            let mut options = ConnectOptions::new(format!(
                "sqlite://{}?mode=rwc",
                tuta_path.as_ref().to_str().unwrap()
            ));
            options.set_application_name(app_name);
            options.sqlcipher_key(hex::encode(&*tuta_passphrase));
            options.max_connections(1);
            let tuta_db = Database::connect(options)
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            tuta_db
                .execute_unprepared(
                    r#"
                        pragma journal_mode = WAL;
                        pragma synchronous = NORMAL;
                    "#,
                )
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            TutaMigrationHandler::up(&tuta_db, None)
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;

            tuta_db
        };

        let email_db = {
            let email_passphrase = get_passphrase(app_name, "email").await?;
            let mut options = ConnectOptions::new(format!(
                "sqlite://{}?mode=rwc",
                email_path.as_ref().to_str().unwrap()
            ));
            options.set_application_name(app_name);
            options.sqlcipher_key(hex::encode(&*email_passphrase));
            options.max_connections(1);
            let email_db = Database::connect(options)
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            email_db
                .execute_unprepared(
                    r#"
                        pragma journal_mode = WAL;
                        pragma synchronous = NORMAL;
                    "#,
                )
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            email_db
        };

        Ok(Self { tuta_db })
    }

    pub async fn sync_tuta_accounts(&self, tuta_accounts: Vec<TutaAccountConfig>) -> Result<()> {
        let transaction = self
            .tuta_db
            .begin()
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        let mut tuta_accounts = tuta_accounts.into_iter().collect::<HashSet<_>>();
        Entity::delete_many()
            .filter(
                tuta_account::Column::Login
                    .is_not_in(tuta_accounts.iter().map(|account| &account.login)),
            )
            .exec(&transaction)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        for account in Entity::find()
            .all(&transaction)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?
        {
            let account = TutaAccountConfig {
                login: account.login,
            };
            tuta_accounts.remove(&account);
        }

        Entity::insert_many(
            tuta_accounts
                .into_iter()
                .map(|account| {
                    let StoreEntry::File(pass_entry) =
                        libpass::retrieve(&format!("ems/tuta/{}/login", account.login))
                            .map_err(|error| raise!(ErrorKind::Pass(error)))?
                    else {
                        throw!(ErrorKind::PassEntryDir);
                    };
                    let password = String::from_utf8(
                        pass_entry
                            .plain_io_ro()
                            .map_err(|error| raise!(ErrorKind::Pass(error)))?
                            .as_ref()
                            .trim_ascii()
                            .to_vec(),
                    )
                    .unwrap();
                    let StoreEntry::File(totp_entry) =
                        libpass::retrieve(&format!("ems/tuta/{}/totp", account.login))
                            .map_err(|error| raise!(ErrorKind::Pass(error)))?
                    else {
                        throw!(ErrorKind::PassEntryDir)
                    };
                    let totp = String::from_utf8(
                        totp_entry
                            .plain_io_ro()
                            .map_err(|error| raise!(ErrorKind::Pass(error)))?
                            .as_ref()
                            .trim_ascii()
                            .to_vec(),
                    )
                    .unwrap();
                    Ok(tuta_account::ActiveModel {
                        login: ActiveValue::Set(account.login),
                        password: ActiveValue::Set(password),
                        totp: ActiveValue::Set(totp),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        )
        .exec(&transaction)
        .await
        .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;

        transaction
            .commit()
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        Ok(())
    }

    pub async fn tuta_accounts(&self) -> Result<Vec<Credentials>> {
        let mut credentials = Vec::new();
        for account in tuta_account::Entity::load()
            .with(tuta_credential::Entity)
            .all(&self.tuta_db)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?
            .into_iter()
        {
            if let Some(creds) = account.credentials.into_option() {
                credentials.push(Credentials {
                    login: creds.login,
                    user_id: GeneratedId(creds.user_id),
                    access_token: creds.access_token,
                    encrypted_passphrase_key: creds.encrypted_passphrase_key,
                    credential_type: creds.credential_type.0,
                })
            } else {
                todo!()
            }
        }
        Ok(credentials)
    }
}
