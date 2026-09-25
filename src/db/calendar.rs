use std::path::Path;
use std::{collections::HashSet, mem::take, sync::Arc};

use chrono::{DateTime, Duration, NaiveDate};
use chrono_tz::Tz;
use emer::{raise, throw};
use fast_dav_rs::CalDavClient;
use icalendar::EventStatus;
use iced::Color;
use libpass::StoreEntry;
use migrations::MigrationHandler;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ExprTrait, IntoActiveModel, QueryFilter,
    TransactionTrait,
};
use sea_orm::{
    ActiveValue, ConnectOptions, ConnectionTrait, Database, DatabaseConnection,
    sea_query::OnConflict,
};
use sea_orm_migration::MigratorTrait;

use super::passphrase::get_passphrase;
use crate::config::calendar::AccountConfig;
use crate::{
    error::{ErrorKind, Result},
    ui::App,
};

mod account;
mod calendar;
mod migrations;
mod object;

#[allow(unused_imports)]
pub use account::{Account, AccountEntity, AccountMetadata};
#[allow(unused_imports)]
pub use calendar::{Calendar, CalendarEntity, CalendarMetadata};
#[allow(unused_imports)]
pub use object::{Object, ObjectEntity, ObjectMetadata};

#[derive(Debug, Clone)]
pub enum TimeSpan {
    FullDay {
        day: NaiveDate,
        /// Duration in days
        duration: u32,
    },
    DateTime {
        start: DateTime<Tz>,
        duration: Duration,
    },
}

#[derive(Debug, Clone)]
pub struct Occurrence {
    pub color: Color,
    pub description: Option<Arc<str>>,
    pub summary: Option<Arc<str>>,
    pub status: Option<EventStatus>,
    pub time_span: TimeSpan,
}

/// The global storage of the application.
#[derive(Debug, Clone)]
pub struct CalendarStorage {
    connection: DatabaseConnection,
}

impl CalendarStorage {
    pub async fn open(path: impl AsRef<Path>, app_name: &str) -> Result<Self> {
        let passphrase = get_passphrase(app_name, "calendar").await?;
        let mut options = ConnectOptions::new(format!(
            "sqlite://{}?mode=rwc",
            path.as_ref().to_str().unwrap()
        ));
        options.set_application_name(app_name);
        options.sqlcipher_key(hex::encode(&*passphrase));
        options.max_connections(1);
        let connection = Database::connect(options)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        connection
            .execute_unprepared(
                r#"
                    pragma journal_mode = WAL;
                    pragma synchronous = NORMAL;
                "#,
            )
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        MigrationHandler::up(&connection, None)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        Ok(Self { connection })
    }

    pub async fn sync_accounts(&self, accounts: Vec<AccountConfig>) -> Result<()> {
        let mut accounts = accounts.into_iter().collect::<HashSet<_>>();
        let mut condition = Condition::all();
        for account in accounts.iter() {
            condition = condition.add(
                account::Column::Server
                    .eq(&account.server)
                    .and(account::Column::Username.eq(&account.username))
                    .not(),
            );
        }
        account::AccountEntity::delete_many()
            .filter(condition)
            .exec(&self.connection)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        for account in account::AccountEntity::find()
            .all(&self.connection)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?
        {
            let account = AccountConfig {
                server: account.server,
                username: account.username,
            };
            accounts.remove(&account);
        }

        account::AccountEntity::insert_many(
            accounts
                .into_iter()
                .map(|account| {
                    let StoreEntry::File(pass_entry) = libpass::retrieve(&format!(
                        "ems/cal/{}/{}",
                        account.server, account.username
                    ))
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
                    Ok(account::ActiveModel {
                        username: ActiveValue::Set(account.username),
                        password: ActiveValue::Set(password),
                        server: ActiveValue::Set(account.server),
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        )
        .exec(&self.connection)
        .await
        .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        Ok(())
    }

    /// Perform an incremental synchronization of all calendars on all accounts.
    pub async fn sync_calendars(&self) -> Result<()> {
        for account in AccountEntity::find()
            .all(&self.connection)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?
        {
            self.sync_calendars_of(account).await?;
        }
        Ok(())
    }

    /// Perform an incremental synchronization of all calendars of `account`.
    pub async fn sync_calendars_of(&self, account: AccountMetadata) -> Result<()> {
        let client = CalDavClient::new(
            &format!("https://{}/remote.php/dav/", account.server),
            Some(&account.username),
            Some(&account.password),
        )
        .map_err(|error| raise!(ErrorKind::Dav(error)))?;

        // The `principal_path` is the "directory" where user stuff lives.  This is akin to a
        // $HOME path, but for the WebDAV protocol.
        let principal_path = client
            .discover_current_user_principal()
            .await
            .map_err(|error| raise!(ErrorKind::Dav(error)))?
            .unwrap();

        let mut discovered_calendars = Vec::new();

        for home in client
            .discover_calendar_home_set(&principal_path)
            .await
            .map_err(|error| raise!(ErrorKind::Dav(error)))?
        {
            // Technically, the user is allowed to have calendars in different `home`s
            // (ie. subdirectories of its `principal_path`), even though in practice, servers
            // store calendars in a single directory.
            discovered_calendars.extend(
                client
                    .list_calendars(&home)
                    .await
                    .map_err(|error| raise!(ErrorKind::Dav(error)))?,
            );
        }
        CalendarEntity::delete_many()
            .filter(calendar::Column::AccountUser.eq(account.username.clone()))
            .filter(calendar::Column::AccountServer.eq(account.server.clone()))
            .filter(
                calendar::Column::Href.is_not_in(
                    discovered_calendars
                        .iter()
                        .map(|calendar| calendar.href.as_str())
                        .collect::<HashSet<_>>(),
                ),
            )
            .exec(&self.connection)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        let calendars = CalendarEntity::insert_many(discovered_calendars.into_iter().map(|cal| {
            calendar::ActiveModel {
                href: ActiveValue::Set(cal.href),
                account_user: ActiveValue::Set(account.username.clone()),
                account_server: ActiveValue::Set(account.server.clone()),
                name: ActiveValue::Set(
                    cal.displayname.unwrap_or_else(|| String::from("(No Name)")),
                ),
                color: ActiveValue::Set(cal.color),
                ..Default::default()
            }
        }))
        .on_conflict(take(
            OnConflict::columns([
                calendar::Column::AccountUser,
                calendar::Column::AccountServer,
                calendar::Column::Href,
            ])
            .update_columns([calendar::Column::Name, calendar::Column::Color]),
        ))
        .exec_with_returning(&self.connection)
        .await
        .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;

        for cal in calendars {
            let delta = client
                .sync_session(&cal.href)
                .with_sync_token(cal.sync_token.as_deref())
                .incremental()
                .await
                .map_err(|_error| todo!())?;
            // We begin a transaction to ensure that the data in our database corresponds
            // exactly to the data in the remote server, up to the point in history defined
            // by the sync token.
            //
            // To do so, we want to ensure that the sync token, as well as local data, gets
            // updated atomically, to avoid a desynchronization.
            let txn = self
                .connection
                .begin()
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;

            // Firstly.  Let's delete objects to be deleted.
            if delta.resynced {
                // For some reason, the server has answer a full synchronization rather than
                // an incremental one.  This can be due to:
                // - internal changes in the serverstructure, making synchronization
                //   sessions meaningless
                // - the token being invalid, either because it was corrupted at some point,
                //   or because the server invalidated it, which can happen when the token
                //   hasn't been used for a long time, or when the server has to serve a lot
                //   of clients.
                // This changes the way we should interpret the result:
                // - only the`added` field will contain any data;
                // - the server expects us to reset all data.
                ObjectEntity::delete_many()
                    .filter(object::Column::AccountUser.eq(account.username.clone()))
                    .filter(object::Column::AccountServer.eq(account.server.clone()))
                    .filter(object::Column::CalendarHref.eq(&cal.href))
                    .exec(&txn)
                    .await
                    .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            } else if !delta.deleted.is_empty() {
                ObjectEntity::delete_many()
                    .filter(object::Column::AccountUser.eq(account.username.clone()))
                    .filter(object::Column::AccountServer.eq(account.server.clone()))
                    .filter(object::Column::CalendarHref.eq(&cal.href))
                    .filter(
                        object::Column::Href.is_in(delta.deleted.iter().collect::<HashSet<_>>()),
                    )
                    .exec(&txn)
                    .await
                    .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            }

            // Secondly.  Let's insert and update new objects.
            ObjectEntity::insert_many(
                delta
                    .added
                    .into_iter()
                    .chain(delta.modified.into_iter())
                    .map(|obj| object::ActiveModel {
                        href: ActiveValue::Set(obj.href),
                        calendar_href: ActiveValue::Set(cal.href.clone()),
                        account_user: ActiveValue::Set(account.username.clone()),
                        account_server: ActiveValue::Set(account.server.clone()),
                        etag: ActiveValue::Set(obj.etag),
                        ical_data: ActiveValue::Set(obj.data.unwrap()),
                    }),
            )
            .on_conflict(take(
                OnConflict::columns([
                    object::Column::AccountUser,
                    object::Column::AccountServer,
                    object::Column::CalendarHref,
                    object::Column::Href,
                ])
                .update_columns([object::Column::Etag, object::Column::IcalData]),
            ))
            .exec(&txn)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            // Finally.  Let's update the token.
            let mut cal = cal.into_active_model();
            cal.sync_token.set_if_not_equals(delta.sync_token);
            cal.save(&txn)
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
            txn.commit()
                .await
                .map_err(|error| raise!(ErrorKind::Sqlite(error)))?;
        }
        Ok(())
    }

    pub async fn calendars(&self) -> Result<Vec<Calendar>> {
        CalendarEntity::load()
            .with(ObjectEntity)
            .all(&self.connection)
            .await
            .map_err(|error| raise!(ErrorKind::Sqlite(error)))
    }

    pub async fn login_tuta(&self) -> Result<()> {
        let http_client = HttpClient::new();
        Ok(())
    }
}
