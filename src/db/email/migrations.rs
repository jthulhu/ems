use async_trait::async_trait;
use migration_001::JmapMigration001;
use sea_orm_migration::prelude::*;

mod migration_001;

pub struct TutaMigrationHandler;

#[async_trait]
impl MigratorTrait for TutaMigrationHandler {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(JmapMigration001)]
    }
}
