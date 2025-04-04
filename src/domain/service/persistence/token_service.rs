use chrono::{DateTime, Utc};
use num_traits::FromPrimitive;
use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token::{self};
use crate::ports::db::repository::{CrudRepository, TokenRepository};
use sea_orm::{DbErr, IntoActiveModel};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::Decimal;
use crate::domain::entity::TokenEntity;
use crate::domain::utils::Converter;

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
        //rounding here is simplified. We are always in USD and it has everywhere 6
        active_model.price = Set(Decimal::from_f64(Converter::round_f64(token_entity.price,6)).unwrap());
        active_model.updated_at = Set(Utc::now().into());
        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }

    pub async fn update_price_change(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let mut active_model: token::ActiveModel = token_entity.to_model().into();
        active_model.price_change24 = Set(Decimal::from_f32(token_entity.price_change24).unwrap());
        active_model.price = Set(Decimal::from_f64(Converter::round_f64(token_entity.price,6)).unwrap());
        active_model.updated_at = Set(Utc::now().into());
        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }

    pub async fn update_volume(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let mut active_model: token::ActiveModel = token_entity.to_model().into();
        active_model.volume24 = Set(Decimal::from_f64(token_entity.volume_24).unwrap());
        active_model.volume24_usd = Set(Decimal::from_f64(token_entity.volume_24_usd).unwrap());
        active_model.updated_at = Set(Utc::now().into());

        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }

    pub async fn update_liquidity(token_entity: TokenEntity) -> Result<TokenEntity, DbErr> {
        let mut active_model: token::ActiveModel = token_entity.to_model().into();
        active_model.liquidity = Set(Decimal::from_f64(token_entity.liquidity).unwrap());
        active_model.no_liquidity = Set(token_entity.no_liquidity);
        active_model.liquidity_usd = Set(Decimal::from_f64(token_entity.liquidity_usd).unwrap());
        active_model.updated_at = Set(Utc::now().into());

        let updated_model = TokenRepository::update(active_model).await?;
        Ok(TokenEntity::from_model(&updated_model))
    }

    /// Returns tokens sorted by price change in ascending order (biggest losers first)
    pub async fn find_biggest_losers() -> Result<Vec<TokenEntity>, DbErr> {
        let models = TokenRepository::find_sorted_by_price_change_asc().await?;
        Ok(models.into_iter().map(|model: token::Model| TokenEntity::from_model(&model)).collect())
    }

    /// Returns tokens sorted by price change in descending order (biggest gainers first)
    pub async fn find_biggest_gainers() -> Result<Vec<TokenEntity>, DbErr> {
        let models = TokenRepository::find_sorted_by_price_change_desc().await?;
        Ok(models.into_iter().map(|model: token::Model| TokenEntity::from_model(&model)).collect())
    }

    /// Returns tokens sorted by trading volume in descending order (highest volume first)
    pub async fn find_highest_volume() -> Result<Vec<TokenEntity>, DbErr> {
        let models = TokenRepository::find_sorted_by_volume_desc().await?;
        Ok(models.into_iter().map(|model: token::Model| TokenEntity::from_model(&model)).collect())
    }
}