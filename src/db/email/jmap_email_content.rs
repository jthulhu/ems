use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmap_email_content")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: String,
    #[sea_orm(primary_key)]
    pub email_id: String,
    pub text_content: Option<String>,
    pub html_content: Option<String>,
    #[sea_orm(belongs_to, from = "(account_id, email_id)", to = "(account_id, id)")]
    pub email: BelongsTo<super::jmap_email::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
