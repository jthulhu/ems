use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "asset")]
pub struct Model {
    #[sea_orm(primary_key)]
    sha256_hash: String,
    size: u32,
    data: Vec<u8>,
}

impl ActiveModelBehavior for ActiveModel {}
