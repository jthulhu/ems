use sea_orm::{TryGetable, prelude::*, sea_query::ValueType};
use sea_orm_migration::prelude::{ArrayType, ValueTypeErr};
use tuta_sdk::{
    GeneratedId,
    login::{CredentialType, Credentials},
};

#[derive(Debug, PartialEq, Clone)]
pub struct CredType(pub CredentialType);

impl Eq for CredType {}

impl From<bool> for CredType {
    fn from(is_internal: bool) -> Self {
        if is_internal {
            CredType(CredentialType::Internal)
        } else {
            CredType(CredentialType::External)
        }
    }
}

impl From<CredType> for bool {
    fn from(value: CredType) -> Self {
        matches!(value.0, CredentialType::Internal)
    }
}

impl TryGetable for CredType {
    fn try_get_by<I: sea_orm::ColIdx>(res: &QueryResult, index: I) -> Result<Self, TryGetError> {
        Ok(res.try_get_by::<bool, _>(index)?.into())
    }
}

impl From<CredType> for Value {
    fn from(value: CredType) -> Self {
        bool::from(value).into()
    }
}

impl ValueType for CredType {
    fn try_from(v: Value) -> Result<Self, ValueTypeErr> {
        Ok(<bool as ValueType>::try_from(v)?.into())
    }

    fn type_name() -> String {
        bool::type_name()
    }

    fn array_type() -> ArrayType {
        bool::array_type()
    }

    fn column_type() -> ColumnType {
        bool::column_type()
    }
}

#[sea_orm::model]
#[derive(Debug, Clone, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "tuta_credential")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: String,
    pub access_token: String,
    pub encrypted_passphrase_key: Vec<u8>,
    pub credential_type: CredType,
    #[sea_orm(unique)]
    pub login: String,
    #[sea_orm(belongs_to, from = "login", to = "login")]
    pub account: BelongsTo<super::tuta_account::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn get_credentials(self) -> Credentials {
        Credentials {
            login: self.login,
            user_id: GeneratedId(self.user_id),
            access_token: self.access_token,
            encrypted_passphrase_key: self.encrypted_passphrase_key,
            credential_type: self.credential_type.0,
        }
    }
}
