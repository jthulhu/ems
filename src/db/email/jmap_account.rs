use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmap_account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub email: String,
    pub provider: String,
    pub password: String,
    pub folder_token: String,
    pub email_token: String,
    #[sea_orm(has_many)]
    pub folders: HasMany<super::jmap_folder::Entity>,
    #[sea_orm(has_many)]
    pub emails: HasMany<super::jmap_email::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
