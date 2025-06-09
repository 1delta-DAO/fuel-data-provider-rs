use chrono::{DateTime, Utc};
use crate::domain::entity::pair_swaps_entity::PairSwapsEntity;
use crate::ports::db::model::pair_swaps::Model;
use sea_orm::DbErr;
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::ports::db::repository::{CrudRepository, PairSwapsRepository};

/// Service layer for handling operations related to `PairSwapsEntity`
pub struct PairSwapsService;

#[allow(dead_code)]
impl PairSwapsService {
    /// Retrieves all records matching the given `pair_id` and converts them to domain entities
    pub async fn get_by_pair_id(pair_id: Uuid) -> Result<Vec<PairSwapsEntity>, DbErr> {
        let models = PairSwapsRepository::find_by_pair_id(pair_id).await?;
        Ok(models.iter().map(PairSwapsEntity::from_model).collect())
    }

    /// Inserts multiple records into the database using domain entities
    pub async fn create_many_with_sync(entities: Vec<PairSwapsEntity>,
                             block_number: i32,
                             block_time: DateTime<Utc>
    ) -> Result<(), DbErr> {
        let active_models: Vec<Model> = entities.into_iter().map(|e| e.to_model()).collect();
        let active_models: Vec<_> = active_models.into_iter().map(Into::into).collect();
        PairSwapsRepository::insert_many_with_sync(active_models,block_number,block_time).await
    }

    /// Retrieves all records from the database and converts them to domain entities
    pub async fn get_all() -> Result<Vec<PairSwapsEntity>, DbErr> {
        let models = PairSwapsRepository::find_all().await?;
        Ok(models.iter().map(PairSwapsEntity::from_model).collect())
    }

    /// Retrieves a single record by its ID and converts it to a domain entity
    pub async fn get_by_id(id: Uuid) -> Result<Option<PairSwapsEntity>, DbErr> {
        match PairSwapsRepository::find_by_id(id).await? {
            Some(model) => Ok(Some(PairSwapsEntity::from_model(&model))),
            None => Ok(None),
        }
    }

    /// Checks if there is at least one record with the given block_number.
    /// Returns `true` if a record exists, `false` otherwise (including error cases).
    pub async fn exists_by_block_number(block_number: i32) -> bool {
        match PairSwapsRepository::find_by_block_number(block_number).await {
            Ok(result) => !result.is_empty(),
            Err(err) => {
                log::error!("Error checking block number existence: {:?}", err);
                false
            }
        }
    }

    /// Deletes a record by its ID
    pub async fn delete(id: Uuid) -> Result<(), DbErr> {
        PairSwapsRepository::delete(id).await
    }

    pub async fn delete_expired() -> Result<u64, DbErr> {
        PairSwapsRepository::delete_expired().await
    }
}