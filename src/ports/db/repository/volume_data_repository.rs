use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;
use crate::ports::db::model::volume_data;
use crate::ports::db::model::volume_data::Model;

pub struct VolumeDataRepository;

#[async_trait]
impl CrudRepository<volume_data::Entity> for VolumeDataRepository {}

impl VolumeDataRepository {
    /// Finds volume records by id
    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Vec<Model>, DbErr> {
        Self::find_by_column_many(volume_data::Column::TokenId, token_id.to_owned()).await
    }
}