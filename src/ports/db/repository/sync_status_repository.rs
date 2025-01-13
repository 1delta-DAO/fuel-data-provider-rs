use crate::ports::db::model::sync_status;
use crate::ports::db::model::sync_status::Model;
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;

pub struct SyncStatusRepository;

#[async_trait]
impl CrudRepository<sync_status::Entity> for SyncStatusRepository {}

impl SyncStatusRepository {
    pub async fn get_status() -> Result<Option<Model>, DbErr> {
        let records = Self::find_all().await?;

        if let Some(record) = records.first() {
            Ok(Option::from(record.clone()))
        } else {
            let new_record = sync_status::ActiveModel {
                id: sea_orm::ActiveValue::Set(Uuid::new_v4()),
                ..Default::default()
            };
            match Self::create(new_record).await {
                Ok(created_record) => Ok(Some(created_record)),
                Err(err) => Err(err),
            }
        }
    }
}
