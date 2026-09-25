use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "email_account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub login: String,
    pub password: String,
    pub imap_url: String,
    pub imap_port: i32,
    pub smtp_url: String,
    pub smtp_port: i32,
    #[sea_orm(has_many)]
    pub folders: HasMany<super::email_folder::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
