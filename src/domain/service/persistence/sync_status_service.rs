use crate::domain::entity::sync_status_entity::SyncStatusEntity;
use sea_orm::DbErr;
use crate::domain::entity::entity::Entity;
use crate::ports::db::repository::SyncStatusRepository;

pub struct SyncStatusService;

#[allow(dead_code)]
impl SyncStatusService{
    pub async fn get_status() -> Result<Option<SyncStatusEntity>, DbErr> {
        match SyncStatusRepository::get_status().await {
            Ok(Some(model)) => Ok(Some(SyncStatusEntity::from_model(&model))),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }

    pub async fn get_block_number() -> Result<Option<i32>, DbErr> {
        match SyncStatusRepository::get_status().await {
            Ok(Some(model)) => Ok(Some(model.block_number)),
            Ok(None) => Ok(None),
            Err(err) => Err(err),
        }
    }
}