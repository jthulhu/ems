use async_trait::async_trait;
use sea_orm::{DbErr, DeriveMigrationName};
use sea_orm_migration::{MigrationTrait, SchemaManager};

#[derive(Debug, DeriveMigrationName)]
pub struct TutaMigration001;

mod tuta_credential {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, IntoIden, Table},
    };
    use sea_orm_migration::SchemaManager;

    use super::tuta_account::TutaAccount;

    #[derive(Debug, DeriveIden)]
    pub enum TutaCredential {
        Table,
        UserId,
        AccessToken,
        EncryptedPassphraseKey,
        CredentialType,
        Login,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TutaCredential::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TutaCredential::UserId).text().not_null())
                    .col(
                        ColumnDef::new(TutaCredential::AccessToken)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TutaCredential::EncryptedPassphraseKey)
                            .blob()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(TutaCredential::Login)
                            .text()
                            .null()
                            .unique_key(),
                    )
                    .primary_key(Index::create().col(TutaCredential::UserId))
                    .foreign_key(
                        ForeignKey::create()
                            .name(format!(
                                "fk-{}-{}",
                                TutaCredential::Table.into_iden(),
                                TutaAccount::Table.into_iden(),
                            ))
                            .from(TutaCredential::Table, TutaCredential::Login)
                            .to(TutaAccount::Table, TutaAccount::Login)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TutaCredential::Table).take())
            .await?;
        Ok(())
    }
}

mod tuta_account {
    use sea_orm::{DbErr, DeriveIden, sea_query::ColumnDef, sea_query::Table};
    use sea_orm_migration::SchemaManager;

    #[derive(Debug, DeriveIden)]
    pub enum TutaAccount {
        Table,
        Login,
        Password,
        Totp,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(TutaAccount::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(TutaAccount::Login).text().not_null())
                    .col(ColumnDef::new(TutaAccount::Password).text().not_null())
                    .col(ColumnDef::new(TutaAccount::Totp).text().not_null())
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(TutaAccount::Table).take())
            .await?;
        Ok(())
    }
}

#[async_trait]
impl MigrationTrait for TutaMigration001 {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        email_account::up(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        email_account::down(manager).await?;
        Ok(())
    }
}

#[derive(Debug, DeriveMigrationName)]
pub struct EmailMigration001;

mod email_account {}
