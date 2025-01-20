use crate::ports::db::model::pair_swaps;
use crate::ports::db::model::pair_swaps::Model;
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{DbErr, EntityTrait, ColumnTrait, QueryFilter};
use uuid::Uuid;

pub struct PairSwapsRepository;

#[async_trait]
impl CrudRepository<pair_swaps::Entity> for PairSwapsRepository {}

impl PairSwapsRepository {
    /// Inserts multiple records into the pair_swaps table
    pub async fn insert_many(records: Vec<pair_swaps::ActiveModel>) -> Result<(), DbErr> {
        Self::create_many(records).await
    }

    /// Finds all records with the given pair_id
    pub async fn find_by_pair_id(pair_id: Uuid) -> Result<Vec<Model>, DbErr> {
        pair_swaps::Entity::find()
            .filter(pair_swaps::Column::PairId.eq(pair_id))
            .all(&crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap())
            .await
    }

    /// Deletes all records where `block_time` is older than the provided timestamp
    pub async fn delete_older_than(timestamp: DateTime<Utc>) -> Result<u64, DbErr> {
        let db = crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap();
        let result = pair_swaps::Entity::delete_many()
            .filter(pair_swaps::Column::BlockTime.lte(timestamp))
            .exec(&db)
            .await?;

        Ok(result.rows_affected)
    }
}