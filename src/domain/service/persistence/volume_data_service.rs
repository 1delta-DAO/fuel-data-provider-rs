use sea_orm::{DbErr, IntoActiveModel};
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::VolumeDataEntity;
use crate::ports::db::repository::{CrudRepository, VolumeDataRepository};

pub struct VolumeDataService;


impl VolumeDataService {
    /// Creates a new price record
    pub async fn create(volume_entity: VolumeDataEntity) -> Result<VolumeDataEntity, DbErr> {
        let model = volume_entity.to_model();
        let created_model = VolumeDataRepository::create(model.into_active_model()).await?;
        Ok(VolumeDataEntity::from_model(&created_model))
    }

    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Vec<VolumeDataEntity>, DbErr> {
        let models = VolumeDataRepository::find_by_token_id(token_id).await?;
        Ok(models.into_iter().map(|model| VolumeDataEntity::from_model(&model)).collect())
    }
}