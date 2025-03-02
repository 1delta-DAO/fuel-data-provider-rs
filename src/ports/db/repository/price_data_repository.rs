use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;
use crate::ports::db::model::price_data;
use crate::ports::db::model::price_data::Model;

pub struct PriceDataRepository;

#[async_trait]
impl CrudRepository<price_data::Entity> for PriceDataRepository {}
impl PriceDataRepository {
    /// Finds prices by id
    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Vec<Model>, DbErr> {
        Self::find_by_column_many(price_data::Column::TokenId, token_id.to_owned()).await
    }
}