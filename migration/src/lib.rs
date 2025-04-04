pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250403_214830_token_new_columns;
mod m20250404_024403_token_liquidity_usd;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250403_214830_token_new_columns::Migration),
            Box::new(m20250404_024403_token_liquidity_usd::Migration),
        ]
    }
}
