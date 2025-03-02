use chrono::{DateTime, FixedOffset, Timelike};
use num_traits::FromPrimitive;
use sea_orm::{DbErr, IntoActiveModel};
use sea_orm::prelude::Decimal;
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::VolumeDataEntity;
use crate::ports::db::model::volume_data;
use crate::ports::db::model::volume_data::Model;
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

        let timestamp_fixed: DateTime<FixedOffset> = volume_entity.timestamp.into();

        if let Some(existing_record) = VolumeDataRepository::find_by_id(timestamp_fixed).await? {

            let mut existing_entity = VolumeDataEntity::from_model(&existing_record);

            existing_entity.volume += volume_entity.volume;

            let updated_model = VolumeDataRepository::update(existing_entity.to_model().into_active_model()).await?;
            Ok(VolumeDataEntity::from_model(&updated_model))
        } else {
            let model = volume_entity.to_model();
            let created_model = VolumeDataRepository::create(model.into_active_model()).await?;
            Ok(VolumeDataEntity::from_model(&created_model))
        }
    }

    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Vec<VolumeDataEntity>, DbErr> {
        let models = VolumeDataRepository::find_by_token_id(token_id).await?;
        Ok(models.into_iter().map(|model| VolumeDataEntity::from_model(&model)).collect())
    }
}