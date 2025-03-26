use chrono::{DateTime, FixedOffset, Timelike};
use sea_orm::{DbErr, IntoActiveModel};
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::VolumeDataEntity;
use crate::ports::db::repository::{CrudRepository, VolumeDataRepository};

pub struct VolumeDataService;


impl VolumeDataService {
    /// Increments the volume for the given token and timestamp (rounded to the nearest minute)
    pub async fn create_or_update(mut volume_entity: VolumeDataEntity) -> Result<VolumeDataEntity, DbErr> {
        volume_entity.timestamp = volume_entity.timestamp
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        //let timestamp_fixed: DateTime<FixedOffset> = volume_entity.timestamp.into();

        //log::info!("COU: Volume record {:?}", volume_entity);

        if let Some(existing_record) = VolumeDataRepository::find_by_timestamp_and_token_id(&volume_entity.timestamp,&volume_entity.token_id).await? {

            let mut existing_entity = VolumeDataEntity::from_model(&existing_record);

            let new_volume = existing_entity.volume + volume_entity.volume;

            /*log::info!("COU: Incremental volume update {} -> {} = {}",
               existing_entity.volume,
               volume_entity.volume,new_volume);*/

            existing_entity.volume = new_volume;


            let updated_model = VolumeDataRepository::update(existing_entity.to_model().into_active_model()).await?;
            Ok(VolumeDataEntity::from_model(&updated_model))
        } else {
            let model = volume_entity.to_model();
            //log::info!("COU: New volume record {:?}", model);
            let created_model = VolumeDataRepository::create(model.into_active_model()).await?;
            Ok(VolumeDataEntity::from_model(&created_model))
        }
    }

    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Option<Vec<VolumeDataEntity>>, DbErr> {
        let models = VolumeDataRepository::find_by_token_id(token_id).await?;

        if let Some(models) = models {
            let entities = models.into_iter().map(|model| VolumeDataEntity::from_model(&model)).collect();
            Ok(Some(entities))
        } else {
            Ok(None)
        }
    }

    /// Finds volume data by timestamp and token_id
    pub async fn find_by_timestamp_and_token_id(
        timestamp: &DateTime<FixedOffset>,
        token_id: &Uuid
    ) -> Result<Option<VolumeDataEntity>, DbErr> {
        // Convert FixedOffset to UTC DateTime for repository call
        let timestamp_utc = timestamp.with_timezone(&chrono::Utc);

        let result = VolumeDataRepository::find_by_timestamp_and_token_id(&timestamp_utc, token_id).await?;

        // Convert the model to entity if found
        Ok(result.map(|model| VolumeDataEntity::from_model(&model)))
    }

    pub async fn delete_expired() -> Result<u64, DbErr> {
        VolumeDataRepository::delete_expired().await
    }
}