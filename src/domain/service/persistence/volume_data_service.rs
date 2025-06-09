use crate::domain::entity::entity::Entity;
use crate::domain::entity::VolumeDataEntity;
use crate::ports::db::repository::{CrudRepository, VolumeDataRepository};
use chrono::Timelike;
use sea_orm::{DbErr, IntoActiveModel};
use uuid::Uuid;

pub struct VolumeDataService;

impl VolumeDataService {
    /// Increments the volume for the given token and timestamp (rounded to the nearest minute)
    pub async fn create_or_update(
        mut volume_entity: VolumeDataEntity,
    ) -> Result<VolumeDataEntity, DbErr> {
        volume_entity.timestamp =
            volume_entity.timestamp.with_second(0).unwrap().with_nanosecond(0).unwrap();

        if let Some(existing_record) = VolumeDataRepository::find_by_timestamp_and_token_id(
            &volume_entity.timestamp,
            &volume_entity.token_id,
        )
        .await?
        {
            let mut existing_entity = VolumeDataEntity::from_model(&existing_record);

            let new_volume = existing_entity.volume + volume_entity.volume;

            existing_entity.volume = new_volume;

            let updated_model =
                VolumeDataRepository::update(existing_entity.to_model().into_active_model())
                    .await?;
            Ok(VolumeDataEntity::from_model(&updated_model))
        } else {
            let model = volume_entity.to_model();
            let created_model = VolumeDataRepository::create(model.into_active_model()).await?;
            Ok(VolumeDataEntity::from_model(&created_model))
        }
    }

    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Option<Vec<VolumeDataEntity>>, DbErr> {
        let models = VolumeDataRepository::find_by_token_id(token_id).await?;

        if let Some(models) = models {
            let entities =
                models.into_iter().map(|model| VolumeDataEntity::from_model(&model)).collect();
            Ok(Some(entities))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_expired() -> Result<u64, DbErr> {
        VolumeDataRepository::delete_expired().await
    }
}
