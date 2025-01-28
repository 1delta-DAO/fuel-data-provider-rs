use crate::ports::db::model::{pair_swaps, sync_status};
use crate::ports::db::model::pair_swaps::Model;
use crate::ports::db::repository::{CrudRepository, SyncStatusRepository};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{DbErr, EntityTrait, ColumnTrait, QueryFilter, TransactionTrait, ActiveValue, IntoActiveModel, ActiveModelTrait};
use sea_orm::ActiveValue::Set;
use uuid::Uuid;

pub struct PairSwapsRepository;

#[async_trait]
impl CrudRepository<pair_swaps::Entity> for PairSwapsRepository {}

impl PairSwapsRepository {
    /// Inserts multiple records with block metadata and updates sync status within a transaction.
    pub async fn insert_many_with_sync(
        records: Vec<pair_swaps::ActiveModel>,
        block_number: i32,
        block_time: DateTime<Utc>,
    ) -> Result<(), DbErr> {
        let db = crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap();

        // Begin a transaction
        let txn = db.begin().await?;

        if records.len()>0{
            // Insert the pair_swaps records within the transaction
            let result = pair_swaps::Entity::insert_many(records)
                .exec(&txn)
                .await;

            if result.is_err() {
                txn.rollback().await?;
                log::info!("ERROR: {:?}",result);
                return Err(result.unwrap_err());
            }

        }

        // Fetch or create SyncStatus record
        let sync_status_model = sync_status::Entity::find().one(&txn).await?;

        let active_model = match sync_status_model {
            Some(model) => model.into_active_model(), // Convert existing to ActiveModel
            None => sync_status::ActiveModel {
                id: ActiveValue::Set(uuid::Uuid::new_v4()),
                ..Default::default()
            },
        };

        // Update sync status columns in ActiveModel
        let mut updated_sync_status = active_model;
        updated_sync_status.block_number = Set(block_number);
        updated_sync_status.block_time = Set(Some(block_time).map(|t| t.into()));
        updated_sync_status.updated_at = Set(Utc::now().into());

        // Save updated SyncStatus within the transaction
        if updated_sync_status.update(&txn).await.is_err() {
            txn.rollback().await?;
            return Err(DbErr::Custom("Failed to update sync status".into()));
        }

        log::info!("Sync status updated: block_number = {}, block_time = {}", block_number, block_time);
        // Commit transaction if all operations are successful
        txn.commit().await?;

        Ok(())
    }

    /// Finds all records with the given pair_id
    pub async fn find_by_pair_id(pair_id: Uuid) -> Result<Vec<Model>, DbErr> {
        pair_swaps::Entity::find()
            .filter(pair_swaps::Column::PairId.eq(pair_id))
            .all(&crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap())
            .await
    }

    /// Finds all records with the given block_number
    pub async fn find_by_block_number(block_number: i32) -> Result<Vec<Model>, DbErr> {
        let db = crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap();

        pair_swaps::Entity::find()
            .filter(pair_swaps::Column::BlockNumber.eq(block_number.to_string()))
            .all(&db)
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