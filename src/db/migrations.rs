use sea_orm::sea_query::IntoIden;

pub fn fk_name(from: impl IntoIden, to: impl IntoIden) -> String {
    format!("fk-{}-{}", from.into_iden(), to.into_iden())
}
