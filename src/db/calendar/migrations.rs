use async_trait::async_trait;
use sea_orm_migration::prelude::*;

mod migration_001;

use migration_001::Migration001;

pub struct MigrationHandler;

#[async_trait]
impl MigratorTrait for MigrationHandler {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(Migration001)]
    }
}
