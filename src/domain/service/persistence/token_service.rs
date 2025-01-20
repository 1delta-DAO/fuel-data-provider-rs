use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token::{self, Model};
use crate::ports::db::repository::{CrudRepository, TokenRepository};
use sea_orm::{DbErr, IntoActiveModel};
use crate::domain::entity::TokenEntity;

pub struct TokenService;

impl TokenService {
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