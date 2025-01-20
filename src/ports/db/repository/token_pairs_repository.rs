use crate::ports::db::model::token_pairs::{self, Model};
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};

pub struct TokenPairsRepository;

#[async_trait]
impl CrudRepository<token_pairs::Entity> for TokenPairsRepository {}

impl TokenPairsRepository {
    /// Finds a token pair by `base_token_details_id` and `quote_token_details_id`
    pub async fn find_by_token_ids(
        base_token_details_id: uuid::Uuid,
        quote_token_details_id: uuid::Uuid,
    ) -> Result<Option<Model>, DbErr> {
        token_pairs::Entity::find()
            .filter(token_pairs::Column::BaseTokenDetailsId.eq(base_token_details_id))
            .filter(token_pairs::Column::QuoteTokenDetailsId.eq(quote_token_details_id))
            .one(&crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap())
            .await
    }
}