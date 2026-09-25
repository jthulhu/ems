use async_trait::async_trait;
use sea_orm_migration::prelude::*;

mod migration_001;

use migration_001::TutaMigration001;

pub struct TutaMigrationHandler;

#[async_trait]
impl MigratorTrait for TutaMigrationHandler {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(TutaMigration001)]
    }
}
