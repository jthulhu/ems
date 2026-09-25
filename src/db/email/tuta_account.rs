use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tuta_account")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub login: String,
    pub password: String,
    pub totp: String,
    #[sea_orm(has_one)]
    pub credentials: HasOne<super::tuta_credential::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
