use sea_orm::prelude::*;

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "email_data")]
pub struct Model {
    /// A meaningless, auto-incremented `id`.
    #[sea_orm(primary_key, auto_increment = true)]
    pub id: i32,

    /// The `id` of the folder where this mail resides.
    #[sea_orm(unique_key = "folder_uid_pair")]
    pub folder_id: i32,

    /// A unique `uid` among emails in parent folder.
    #[sea_orm(unique_key = "folder_uid_pair")]
    pub uid: i32,
    pub modseq: i32,

    /// The `Message-ID` header, used to group mails into conversations.
    pub message_id: Option<String>,

    /// The `\Read` flag.
    pub is_read: bool,
    /// The `\`
    pub is_starred: bool,
    #[sea_orm(belongs_to, from = "folder_id", to = "id")]
    pub folder: BelongsTo<super::email_folder::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
