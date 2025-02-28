use crate::ports::db::model::token;
use crate::ports::db::model::token::Model;
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, QueryFilter, DbErr, EntityTrait};

pub struct TokenRepository;

#[async_trait]
impl CrudRepository<token::Entity> for TokenRepository {}

impl TokenRepository {
    /// Finds a token by its address
    pub async fn find_by_address(address: &str) -> Result<Option<Model>, DbErr> {
        Self::find_by_column(token::Column::Address, address.to_string()).await
    }

    pub async fn find_by_created_between(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<Model>, DbErr> {
        let db = &crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap();
        log::info!("Fetching tokens between {} and {}", start, end);
        token::Entity::find()
            .filter(token::Column::CreatedAt.between(start,end))
            .all(db)
            .await
    }
}