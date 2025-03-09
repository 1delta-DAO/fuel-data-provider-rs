use chrono::{DateTime, Utc};
use num_traits::FromPrimitive;
use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token::{self};
use crate::ports::db::repository::{CrudRepository, TokenRepository};
use sea_orm::{DbErr, IntoActiveModel};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::Decimal;
use crate::domain::entity::TokenEntity;

pub struct TokenService;

#[allow(dead_code)]
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

    /// Finds multiple tokens by a list of addresses
    pub async fn find_by_addresses(addresses: Vec<String>) -> Result<Vec<TokenEntity>, DbErr> {
        let models = TokenRepository::find_by_addresses(addresses).await?;
        Ok(models.into_iter().map(|model: token::Model| TokenEntity::from_model(&model)).collect())
    }

    /// Creates a new token
    pub async fn create(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let model = token_entity.to_model();
        let created_model = TokenRepository::create(model.into_active_model()).await?;
        Ok(TokenEntity::from_model(&created_model))
    }

    /// Updates an existing token
    pub async fn update_price(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let mut active_model: token::ActiveModel = token_entity.to_model().into();
        active_model.price = Set(Decimal::from_f64(token_entity.price).unwrap());
        active_model.updated_at = Set(Utc::now().into());
        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }

    pub async fn update_volume(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let mut active_model: token::ActiveModel = token_entity.to_model().into();
        active_model.volume24 = Set(Decimal::from_f64(token_entity.volume_24).unwrap());
        active_model.updated_at = Set(Utc::now().into());

        log::info!("Active Model: {:?}", active_model);
        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }
}