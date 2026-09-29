use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "jmap_folder")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub account_id: String,
    #[sea_orm(primary_key)]
    pub id: String,
    pub parent_id: Option<String>,
    pub name: String,
    pub role: Option<String>,
    #[sea_orm(belongs_to, from = "account_id", to = "email")]
    pub account: BelongsTo<super::jmap_account::Entity>,
    #[sea_orm(
        self_ref,
        relation_enum = "Parent",
        relation_reverse = "Children",
        from = "(account_id, parent_id)",
        to = "(account_id, id)"
    )]
    pub parent: BelongsTo<Option<Entity>>,
    #[sea_orm(self_ref, relation_enum = "Children", relation_reverse = "Parent")]
    pub children: HasMany<Entity>,
    #[sea_orm(has_many, via = "jmap_email_in_folder")]
    pub emails: HasMany<super::jmap_email::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
