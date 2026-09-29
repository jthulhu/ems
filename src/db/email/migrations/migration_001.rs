use async_trait::async_trait;
use sea_orm::{DbErr, DeriveMigrationName};
use sea_orm_migration::{MigrationTrait, SchemaManager};

mod jmap_account {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    #[derive(Debug, DeriveIden)]
    pub enum JmapAccount {
        Table,
        Email,
        Provider,
        Password,
        FolderToken,
        EmailToken,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapAccount::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(JmapAccount::Email).text().not_null())
                    .col(ColumnDef::new(JmapAccount::Provider).text().not_null())
                    .col(ColumnDef::new(JmapAccount::Password).text().not_null())
                    .col(ColumnDef::new(JmapAccount::FolderToken).text().not_null())
                    .col(ColumnDef::new(JmapAccount::EmailToken).text().not_null())
                    .primary_key(Index::create().col(JmapAccount::Email))
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(JmapAccount::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_folder {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::jmap_account::JmapAccount;

    #[derive(Debug, DeriveIden)]
    pub enum JmapFolder {
        Table,
        AccountId,
        Id,
        ParentId,
        Name,
        Role,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapFolder::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(JmapFolder::AccountId).text().not_null())
                    .col(ColumnDef::new(JmapFolder::Id).text().not_null())
                    .col(ColumnDef::new(JmapFolder::ParentId).text().null())
                    .col(ColumnDef::new(JmapFolder::Name).text().not_null())
                    .col(ColumnDef::new(JmapFolder::Role).text().null())
                    .primary_key(
                        Index::create()
                            .col(JmapFolder::AccountId)
                            .col(JmapFolder::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapFolder::Table, JmapAccount::Table))
                            .from(JmapFolder::Table, JmapFolder::AccountId)
                            .to(JmapAccount::Table, JmapAccount::Email)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapFolder::Table, JmapFolder::Table))
                            .from(
                                JmapFolder::Table,
                                (JmapFolder::AccountId, JmapFolder::ParentId),
                            )
                            .to(JmapFolder::Table, (JmapFolder::AccountId, JmapFolder::Id))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::SetNull),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(take(
                Index::create()
                    .name("index-folder-account")
                    .table(JmapFolder::Table)
                    .col(JmapFolder::AccountId),
            ))
            .await?;
        manager
            .create_index(take(
                Index::create()
                    .name("index-folder-parent")
                    .table(JmapFolder::Table)
                    .col(JmapFolder::AccountId)
                    .col(JmapFolder::ParentId),
            ))
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager.drop_index(take(
            Index::drop()
                .name("index-folder-parent")
                .table(JmapFolder::Table),
        ));
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-folder-account")
                    .table(JmapFolder::Table),
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(JmapFolder::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_email {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::jmap_account::JmapAccount;

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmail {
        Table,
        AccountId,
        Id,
        RawId,
        ThreadId,
        Subject,
        Preview,
        ReceivedAt,
        SentAt,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmail::Table)
                    .col(ColumnDef::new(JmapEmail::AccountId).text().not_null())
                    .col(ColumnDef::new(JmapEmail::Id).text().not_null())
                    .col(ColumnDef::new(JmapEmail::RawId).text().not_null())
                    .col(ColumnDef::new(JmapEmail::ThreadId).text().not_null())
                    .col(ColumnDef::new(JmapEmail::Subject).text().null())
                    .col(ColumnDef::new(JmapEmail::Preview).text().not_null())
                    .col(ColumnDef::new(JmapEmail::ReceivedAt).integer().not_null())
                    .col(ColumnDef::new(JmapEmail::SentAt).integer().null())
                    .primary_key(Index::create().col(JmapEmail::AccountId).col(JmapEmail::Id))
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmail::Table, JmapAccount::Table))
                            .from(JmapEmail::Table, JmapEmail::AccountId)
                            .to(JmapAccount::Table, JmapAccount::Email),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(take(
                Index::create()
                    .name("index-email-account")
                    .table(JmapEmail::Table)
                    .col(JmapEmail::AccountId),
            ))
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email-account")
                    .table(JmapEmail::Table),
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(JmapEmail::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_email_in_folder {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::{jmap_email::JmapEmail, jmap_folder::JmapFolder};

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmailInFolder {
        Table,
        AccountId,
        FolderId,
        EmailId,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmailInFolder::Table)
                    .col(
                        ColumnDef::new(JmapEmailInFolder::AccountId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailInFolder::FolderId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JmapEmailInFolder::EmailId).text().not_null())
                    .primary_key(
                        Index::create()
                            .col(JmapEmailInFolder::AccountId)
                            .col(JmapEmailInFolder::FolderId)
                            .col(JmapEmailInFolder::EmailId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailInFolder::Table, JmapFolder::Table))
                            .from(
                                JmapEmailInFolder::Table,
                                (JmapEmailInFolder::AccountId, JmapEmailInFolder::FolderId),
                            )
                            .to(JmapFolder::Table, (JmapFolder::AccountId, JmapFolder::Id))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailInFolder::Table, JmapEmail::Table))
                            .from(
                                JmapEmailInFolder::Table,
                                (JmapEmailInFolder::AccountId, JmapEmailInFolder::EmailId),
                            )
                            .to(JmapEmail::Table, (JmapEmail::AccountId, JmapEmail::Id))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_in_folder-folder")
                    .table(JmapEmailInFolder::Table)
                    .col(JmapEmailInFolder::AccountId)
                    .col(JmapEmailInFolder::FolderId)
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_in_folder-email")
                    .table(JmapEmailInFolder::Table)
                    .col(JmapEmailInFolder::AccountId)
                    .col(JmapEmailInFolder::EmailId)
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_in_folder-email")
                    .table(JmapEmailInFolder::Table),
            ))
            .await?;
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_in_folder-folder")
                    .table(JmapEmailInFolder::Table),
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(JmapEmailInFolder::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_email_has_keyword_in_folder {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::jmap_email_in_folder::JmapEmailInFolder;

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmailHasKeywordInFolder {
        Table,
        AccountId,
        EmailId,
        FolderId,
        Keyword,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmailHasKeywordInFolder::Table)
                    .col(
                        ColumnDef::new(JmapEmailHasKeywordInFolder::AccountId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasKeywordInFolder::EmailId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasKeywordInFolder::FolderId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasKeywordInFolder::Keyword)
                            .text()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(JmapEmailHasKeywordInFolder::AccountId)
                            .col(JmapEmailHasKeywordInFolder::EmailId)
                            .col(JmapEmailHasKeywordInFolder::FolderId)
                            .col(JmapEmailHasKeywordInFolder::Keyword),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(
                                JmapEmailHasKeywordInFolder::Table,
                                JmapEmailInFolder::Table,
                            ))
                            .from(
                                JmapEmailHasKeywordInFolder::Table,
                                (
                                    JmapEmailHasKeywordInFolder::AccountId,
                                    JmapEmailHasKeywordInFolder::EmailId,
                                    JmapEmailHasKeywordInFolder::FolderId,
                                ),
                            )
                            .to(
                                JmapEmailInFolder::Table,
                                (
                                    JmapEmailInFolder::AccountId,
                                    JmapEmailInFolder::EmailId,
                                    JmapEmailInFolder::FolderId,
                                ),
                            )
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_has_keyword_in_folder-email_in_folder")
                    .table(JmapEmailHasKeywordInFolder::Table)
                    .col(JmapEmailHasKeywordInFolder::AccountId)
                    .col(JmapEmailHasKeywordInFolder::EmailId)
                    .col(JmapEmailHasKeywordInFolder::FolderId)
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_has_keyword_in_folder-email_in_folder")
                    .table(JmapEmailHasKeywordInFolder::Table),
            ))
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(JmapEmailHasKeywordInFolder::Table)
                    .take(),
            )
            .await?;
        Ok(())
    }
}

mod jmap_email_has_address {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::jmap_email::JmapEmail;

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmailHasAddress {
        Table,
        AccountId,
        EmailId,
        Type,
        DisplayName,
        EmailAddress,
        Position,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmailHasAddress::Table)
                    .col(
                        ColumnDef::new(JmapEmailHasAddress::AccountId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasAddress::EmailId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JmapEmailHasAddress::Type).text().not_null())
                    .col(
                        ColumnDef::new(JmapEmailHasAddress::DisplayName)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasAddress::EmailAddress)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasAddress::Position)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(JmapEmailHasAddress::AccountId)
                            .col(JmapEmailHasAddress::EmailId)
                            .col(JmapEmailHasAddress::Type)
                            .col(JmapEmailHasAddress::Position),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailHasAddress::Table, JmapEmail::Table))
                            .from(
                                JmapEmailHasAddress::Table,
                                (JmapEmailHasAddress::AccountId, JmapEmailHasAddress::EmailId),
                            )
                            .to(JmapEmail::Table, (JmapEmail::AccountId, JmapEmail::Id))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_has_address-email")
                    .table(JmapEmailHasAddress::Table)
                    .col(JmapEmailHasAddress::AccountId)
                    .col(JmapEmailHasAddress::EmailId)
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_has_address-email")
                    .table(JmapEmailHasAddress::Table),
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(JmapEmailHasAddress::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_email_content {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::jmap_email::JmapEmail;

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmailContent {
        Table,
        AccountId,
        EmailId,
        TextContent,
        HtmlContent,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmailContent::Table)
                    .col(
                        ColumnDef::new(JmapEmailContent::AccountId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JmapEmailContent::EmailId).text().not_null())
                    .col(ColumnDef::new(JmapEmailContent::TextContent).text().null())
                    .col(ColumnDef::new(JmapEmailContent::HtmlContent).text().null())
                    .primary_key(
                        Index::create()
                            .col(JmapEmailContent::AccountId)
                            .col(JmapEmailContent::EmailId),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailContent::Table, JmapEmail::Table))
                            .from(
                                JmapEmailContent::Table,
                                (JmapEmailContent::AccountId, JmapEmailContent::EmailId),
                            )
                            .to(JmapEmail::Table, (JmapEmail::AccountId, JmapEmail::Id))
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
            .drop_table(Table::drop().table(JmapEmailContent::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_email_attachment {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::jmap_email::JmapEmail;

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmailAttachment {
        Table,
        AccountId,
        EmailId,
        Id,
        Name,
        MimeType,
        Size,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmailAttachment::Table)
                    .col(
                        ColumnDef::new(JmapEmailAttachment::AccountId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailAttachment::EmailId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JmapEmailAttachment::Id).text().not_null())
                    .col(ColumnDef::new(JmapEmailAttachment::Name).text().null())
                    .col(
                        ColumnDef::new(JmapEmailAttachment::MimeType)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailAttachment::Size)
                            .integer()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(JmapEmailAttachment::AccountId)
                            .col(JmapEmailAttachment::EmailId)
                            .col(JmapEmailAttachment::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailAttachment::Table, JmapEmail::Table))
                            .from(
                                JmapEmailAttachment::Table,
                                (JmapEmailAttachment::AccountId, JmapEmailAttachment::EmailId),
                            )
                            .to(JmapEmail::Table, (JmapEmail::AccountId, JmapEmail::Id))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_attachment-email")
                    .table(JmapEmailAttachment::Table)
                    .col(JmapEmailAttachment::AccountId)
                    .col(JmapEmailAttachment::EmailId)
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_attachment-email")
                    .table(JmapEmailAttachment::Table),
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(JmapEmailAttachment::Table).take())
            .await?;
        Ok(())
    }
}

mod asset {
    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    #[derive(Debug, DeriveIden)]
    pub enum Asset {
        Table,
        #[sea_orm(iden = "sha256_hash")]
        Sha256Hash,
        Size,
        Data,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Asset::Table)
                    .col(ColumnDef::new(Asset::Sha256Hash).text().not_null())
                    .col(ColumnDef::new(Asset::Size).integer().not_null())
                    .col(ColumnDef::new(Asset::Data).blob().not_null())
                    .primary_key(Index::create().col(Asset::Sha256Hash))
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Asset::Table).take())
            .await?;
        Ok(())
    }
}

mod jmap_email_has_asset {
    use std::mem::take;

    use sea_orm::{
        DbErr, DeriveIden,
        sea_query::{ColumnDef, ForeignKey, ForeignKeyAction, Index, Table},
    };
    use sea_orm_migration::SchemaManager;

    use crate::db::migrations::fk_name;

    use super::{asset::Asset, jmap_email::JmapEmail};

    #[derive(Debug, DeriveIden)]
    pub enum JmapEmailHasAsset {
        Table,
        AccountId,
        EmailId,
        Id,
        Cid,
        AssetHash,
        MimeType,
    }

    pub async fn up(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(JmapEmailHasAsset::Table)
                    .col(
                        ColumnDef::new(JmapEmailHasAsset::AccountId)
                            .text()
                            .not_null(),
                    )
                    .col(ColumnDef::new(JmapEmailHasAsset::EmailId).text().not_null())
                    .col(ColumnDef::new(JmapEmailHasAsset::Id).text().not_null())
                    .col(ColumnDef::new(JmapEmailHasAsset::Cid).text().null())
                    .col(
                        ColumnDef::new(JmapEmailHasAsset::AssetHash)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(JmapEmailHasAsset::MimeType)
                            .text()
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(JmapEmailHasAsset::AccountId)
                            .col(JmapEmailHasAsset::EmailId)
                            .col(JmapEmailHasAsset::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailHasAsset::Table, JmapEmail::Table))
                            .from(
                                JmapEmailHasAsset::Table,
                                (JmapEmailHasAsset::AccountId, JmapEmailHasAsset::EmailId),
                            )
                            .to(JmapEmail::Table, (JmapEmail::AccountId, JmapEmail::Id))
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name(fk_name(JmapEmailHasAsset::Table, Asset::Table))
                            .from(JmapEmailHasAsset::Table, JmapEmailHasAsset::AssetHash)
                            .to(Asset::Table, Asset::Sha256Hash)
                            .on_update(ForeignKeyAction::Cascade)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_has_asset-email")
                    .table(JmapEmailHasAsset::Table)
                    .col(JmapEmailHasAsset::AccountId)
                    .col(JmapEmailHasAsset::EmailId)
                    .take(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("index-email_has_asset-asset")
                    .table(JmapEmailHasAsset::Table)
                    .col(JmapEmailHasAsset::AssetHash)
                    .take(),
            )
            .await?;
        Ok(())
    }

    pub async fn down(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_has_asset-asset")
                    .table(JmapEmailHasAsset::Table),
            ))
            .await?;
        manager
            .drop_index(take(
                Index::drop()
                    .name("index-email_has_asset-email")
                    .table(JmapEmailHasAsset::Table),
            ))
            .await?;
        manager
            .drop_table(Table::drop().table(JmapEmailHasAsset::Table).take())
            .await?;
        Ok(())
    }
}

#[derive(Debug, DeriveMigrationName)]
pub struct JmapMigration001;

#[async_trait]
impl MigrationTrait for JmapMigration001 {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        jmap_account::up(manager).await?;
        jmap_folder::up(manager).await?;
        jmap_email::up(manager).await?;
        jmap_email_in_folder::up(manager).await?;
        jmap_email_has_keyword_in_folder::up(manager).await?;
        jmap_email_has_address::up(manager).await?;
        jmap_email_content::up(manager).await?;
        jmap_email_attachment::up(manager).await?;
        asset::up(manager).await?;
        jmap_email_has_asset::up(manager).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        jmap_email_has_asset::down(manager).await?;
        asset::down(manager).await?;
        jmap_email_attachment::down(manager).await?;
        jmap_email_content::down(manager).await?;
        jmap_email_has_address::down(manager).await?;
        jmap_email_has_keyword_in_folder::down(manager).await?;
        jmap_email_in_folder::down(manager).await?;
        jmap_email::down(manager).await?;
        jmap_folder::down(manager).await?;
        jmap_account::down(manager).await?;
        Ok(())
    }
}
