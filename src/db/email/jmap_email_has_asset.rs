use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmap_email_has_asset")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: String,
    #[sea_orm(primary_key)]
    pub email_id: String,
    #[sea_orm(primary_key)]
    pub id: String,
    pub cid: Option<String>,
    pub asset_hash: String,
    pub mime_type: String,
    #[sea_orm(belongs_to, from = "asset_hash", to = "sha256_hash")]
    pub asset: BelongsTo<super::asset::Entity>,
    #[sea_orm(belongs_to, from = "(account_id, email_id)", to = "(account_id, id)")]
    pub email: BelongsTo<super::jmap_email::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
