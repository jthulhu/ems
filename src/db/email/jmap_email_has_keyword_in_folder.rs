use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmap_email_has_keyword_in_folder")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: String,
    #[sea_orm(primary_key)]
    pub email_id: String,
    #[sea_orm(primary_key)]
    pub folder_id: String,
    #[sea_orm(primary_key)]
    pub keyword: String,
    #[sea_orm(
        belongs_to,
        from = "(account_id, email_id, folder_id)",
        to = "(account_id, email_id, folder_id)"
    )]
    pub email_in_folder: BelongsTo<super::jmap_email_in_folder::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
