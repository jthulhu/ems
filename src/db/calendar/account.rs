use sea_orm::prelude::*;

pub type AccountEntity = Entity;
#[allow(non_upper_case_globals, unused)]
pub const AccountEntity: AccountEntity = Entity;
#[allow(unused)]
pub type Account = ModelEx;
pub type AccountMetadata = Model;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "calendar_account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub username: String,
    #[sea_orm(primary_key)]
    pub server: String,
    pub password: String,
    #[sea_orm(has_many)]
    pub calendars: HasMany<super::calendar::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
