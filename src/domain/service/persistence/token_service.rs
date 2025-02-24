use chrono::{DateTime, Utc};
use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token::{self};
use crate::ports::db::repository::{CrudRepository, TokenRepository};
use sea_orm::{DbErr, IntoActiveModel};
use crate::domain::entity::TokenEntity;
use crate::ports::db;

pub struct TokenService;

impl TokenService {

    /// Finds all tokens created between two timestamps
    pub async fn find_by_created_between(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<Vec<TokenEntity>, DbErr> {
        let models = TokenRepository::find_by_created_between(start, end).await?;
        Ok(models.into_iter().map(|model: token::Model| TokenEntity::from_model(&model)).collect())
    }

    /// Finds all tokens
    pub async fn find_all_tokens() -> Result<Vec<TokenEntity>, DbErr> {
        let models = TokenRepository::find_all().await?;
        Ok(models.into_iter().map(|model: token::Model| TokenEntity::from_model(&model)).collect())
    }

    /// Finds a token by its address
    pub async fn find_by_address(address: &str) -> Result<Option<TokenEntity>, DbErr> {
        if let Some(model) = TokenRepository::find_by_address(address).await? {
            Ok(Some(TokenEntity::from_model(&model)))
        } else {
            Ok(None)
        }
    }

    /// Creates a new token
    pub async fn create(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let model = token_entity.to_model();
        let created_model = TokenRepository::create(model.into_active_model()).await?;
        Ok(TokenEntity::from_model(&created_model))
    }

    /// Updates an existing token
    pub async fn update(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let active_model: token::ActiveModel = token_entity.to_model().into();
        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }
}