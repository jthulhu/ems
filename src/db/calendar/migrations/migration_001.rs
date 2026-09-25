use async_trait::async_trait;
use sea_orm::{DbErr, DeriveMigrationName};
use sea_orm_migration::{MigrationTrait, SchemaManager};

#[derive(Debug, DeriveMigrationName)]
pub struct Migration001;

mod calendar_objects {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, IntoIden, Table},
    };
    use sea_orm_migration::SchemaManager;

    use super::calendar::Calendar;

    #[derive(Debug, DeriveIden)]
    pub enum CalendarObject {
        Table,
        AccountUser,
        AccountServer,
        Href,
        CalendarHref,
        Etag,
        IcalData,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CalendarObject::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CalendarObject::Href).text().not_null())
                    .col(
                        ColumnDef::new(CalendarObject::AccountUser)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CalendarObject::AccountServer)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CalendarObject::CalendarHref)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(CalendarObject::Etag).text().not_null())
                    .col(ColumnDef::new(CalendarObject::IcalData).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(CalendarObject::Href)
                            .col(CalendarObject::CalendarHref)
                            .col(CalendarObject::AccountUser)
                            .col(CalendarObject::AccountServer),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(format!(
                                "fk-{}-{}",
                                CalendarObject::Table.into_iden(),
                                Calendar::Table.into_iden()
                            ))
                            .from(
                                CalendarObject::Table,
                                (
                                    CalendarObject::AccountUser,
                                    CalendarObject::AccountServer,
                                    CalendarObject::CalendarHref,
                                ),
                            )
                            .to(
                                Calendar::Table,
                                (
                                    Calendar::AccountUser,
                                    Calendar::AccountServer,
                                    Calendar::Href,
                                ),
                            )
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CalendarObject::Table).take())
            .await?;
        Ok(())
    }
}

mod calendar {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, IntoIden, Table},
    };
    use sea_orm_migration::SchemaManager;

    use super::calendar_account::CalendarAccount;

    #[derive(Debug, DeriveIden)]
    pub enum Calendar {
        Table,
        Href,
        SyncToken,
        Name,
        Color,
        AccountUser,
        AccountServer,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Calendar::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Calendar::Href).text().not_null())
                    .col(ColumnDef::new(Calendar::AccountUser).text().not_null())
                    .col(ColumnDef::new(Calendar::AccountServer).text().not_null())
                    .col(ColumnDef::new(Calendar::Name).text().not_null())
                    .col(ColumnDef::new(Calendar::Color).text().null())
                    .col(ColumnDef::new(Calendar::SyncToken).text().null())
                    .primary_key(
                        Index::create()
                            .col(Calendar::Href)
                            .col(Calendar::AccountUser)
                            .col(Calendar::AccountServer),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(format!(
                                "fk-{}-{}",
                                Calendar::Table.into_iden(),
                                CalendarAccount::Table.into_iden(),
                            ))
                            .from(
                                Calendar::Table,
                                (Calendar::AccountUser, Calendar::AccountServer),
                            )
                            .to(
                                CalendarAccount::Table,
                                (CalendarAccount::Username, CalendarAccount::Server),
                            )
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Calendar::Table).take())
            .await?;
        Ok(())
    }
}

mod calendar_account {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    #[derive(Debug, DeriveIden)]
    pub enum CalendarAccount {
        Table,
        Username,
        Password,
        Server,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CalendarAccount::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(CalendarAccount::Username).text().not_null())
                    .col(ColumnDef::new(CalendarAccount::Password).text().not_null())
                    .col(ColumnDef::new(CalendarAccount::Server).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(CalendarAccount::Username)
                            .col(CalendarAccount::Server),
                    )
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CalendarAccount::Table).take())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl MigrationTrait for Migration001 {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        calendar::up(manager).await?;
        calendar_objects::up(manager).await?;
        calendar_account::up(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        calendar_account::down(manager).await?;
        calendar_objects::down(manager).await?;
        calendar::down(manager).await?;
        Ok(())
    }
}
