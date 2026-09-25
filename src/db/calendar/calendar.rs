#![allow(unused)]

use sea_orm::prelude::*;

pub type CalendarEntity = Entity;
#[allow(non_upper_case_globals)]
pub const CalendarEntity: CalendarEntity = Entity;
pub type CalendarMetadata = Model;
pub type Calendar = ModelEx;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "calendar")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub href: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub account_user: String,
    #[sea_orm(primary_key, auto_increment = false, column_type = "Text")]
    pub account_server: String,
    pub sync_token: Option<String>,
    pub name: String,
    pub color: Option<String>,
    #[sea_orm(
        belongs_to,
        from = "(account_user, account_server)",
        to = "(username, server)"
    )]
    pub account: BelongsTo<super::account::Entity>,
    #[sea_orm(has_many)]
    pub objects: HasMany<super::object::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
