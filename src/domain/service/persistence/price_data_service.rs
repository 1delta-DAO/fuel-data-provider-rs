use sea_orm::{DbErr, IntoActiveModel};
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::PriceDataEntity;
use crate::ports::db::repository::{CrudRepository, PriceDataRepository};

pub struct PriceDataService;


impl PriceDataService {

    /// Creates a new price record
    pub async fn create(price_entity: PriceDataEntity) -> Result<PriceDataEntity, DbErr> {
        let model = price_entity.to_model();
        let created_model = PriceDataRepository::create(model.into_active_model()).await?;
        Ok(PriceDataEntity::from_model(&created_model))
    }

    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Vec<PriceDataEntity>, DbErr> {
        let models = PriceDataRepository::find_by_token_id(token_id).await?;
        Ok(models.into_iter().map(|model| PriceDataEntity::from_model(&model)).collect())
    }

    pub async fn delete_expired() -> Result<u64, DbErr> {
        PriceDataRepository::delete_expired().await
    }
}