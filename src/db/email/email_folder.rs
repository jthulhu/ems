use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "email_folder")]
pub struct Model {
    /// A meaningless, auto-incremented `id`.
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,

    /// This data is valid as long as the server's `uid_validity` is the same as the stored one.
    /// If we get a mismatch on sync, this means the server has changed its inner structure,
    /// invalidating the local data.  The only safe thing to do, in this case, is to drop the
    /// data we have on disk, and re-sync.
    pub uid_validity: i32,
    /// The `CONDSTORE` counter, used to perform incremental sync.
    pub highest_modseq: u64,
    pub name: String,
    pub email_login: String,
    pub parent_folder: Option<i32>,
    #[sea_orm(
        self_ref,
        relation_enum = "Parent",
        relation_reverse = "Children",
        from = "parent_folder",
        to = "id"
    )]
    pub parent: BelongsTo<Option<Entity>>,
    #[sea_orm(self_ref, relation_enum = "Children", relation_reverse = "Parent")]
    pub children: HasMany<Entity>,
    #[sea_orm(belongs_to, from = "email_login", to = "login")]
    pub emails: BelongsTo<super::email_account::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
