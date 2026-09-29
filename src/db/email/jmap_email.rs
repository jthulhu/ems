use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmap_email")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: String,
    #[sea_orm(primary_key)]
    pub id: String,
    pub raw_id: String,
    pub thread_id: String,
    pub subject: Option<String>,
    pub preview: String,
    pub received_at: u32,
    pub sent_at: Option<u32>,
    #[sea_orm(belongs_to, from = "account_id", to = "email")]
    pub account: BelongsTo<super::jmap_account::Entity>,
    #[sea_orm(has_many)]
    pub addresses: HasMany<super::jmap_email_has_address::Entity>,
    #[sea_orm(has_one)]
    pub content: HasOne<super::jmap_email_content::Entity>,
    #[sea_orm(has_many)]
    pub attachments: HasMany<super::jmap_email_attachment::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
