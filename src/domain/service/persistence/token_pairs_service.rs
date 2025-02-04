use crate::ports::db::repository::{CrudRepository, TokenPairsRepository};
use sea_orm::{DbErr, IntoActiveModel};
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::{TokenEntity, TokenPairsEntity};

pub struct TokenPairsService;

impl TokenPairsService {
    /// Finds a token pair by `base_token_details_id` and `quote_token_details_id`
    pub async fn find_by_token_ids(
        base_token_details_id: uuid::Uuid,
        quote_token_details_id: uuid::Uuid,
    ) -> Result<Option<TokenPairsEntity>, DbErr> {
        if let Some(model) = TokenPairsRepository::find_by_token_ids(base_token_details_id, quote_token_details_id).await? {
            Ok(Some(TokenPairsEntity::from_model(&model)))
        } else {
            Ok(None)
        }
    }

    /// Finds a token pair by `id`
    pub async fn find_by_id(id: Uuid) -> Result<Option<TokenPairsEntity>, DbErr> {
        if let Some(model) = TokenPairsRepository::find_by_id(id).await? {
            Ok(Some(TokenPairsEntity::from_model(&model)))
        } else {
            Ok(None)
        }
    }

    pub async fn find_or_create_pair(
        base_token: &TokenEntity,
        quote_token: &TokenEntity,
    ) -> Result<TokenPairsEntity, DbErr> {
        // Check if the pair exists
        if let Some(existing_pair) =
            TokenPairsRepository::find_by_token_ids(base_token.id, quote_token.id).await?
        {
            return Ok(TokenPairsEntity::from_model(&existing_pair));
        }

        // Create a new token pair
        let new_pair = TokenPairsEntity {
            id: uuid::Uuid::new_v4(),
            base_token_details_id: base_token.id,
            quote_token_details_id: quote_token.id,
            base_address: base_token.address.clone(),
            quote_address: quote_token.address.clone(),
            pair_address: format!("{}_{}", base_token.address, quote_token.address), // Example pair address
            base_symbol: base_token.symbol.clone(),
            quote_symbol: quote_token.symbol.clone(),
        };

        // Save the new pair
        let created_model = TokenPairsRepository::create(new_pair.to_model().into_active_model()).await?;
        Ok(TokenPairsEntity::from_model(&created_model))
    }

    /// Creates a new token pair
    pub async fn create(token_pair: TokenPairsEntity) -> Result<TokenPairsEntity, DbErr> {
        let model = token_pair.to_model();
        let created_model = TokenPairsRepository::create(model.into_active_model()).await?;
        Ok(TokenPairsEntity::from_model(&created_model))
    }

    /// Updates an existing token pair
    pub async fn update(token_pair: TokenPairsEntity) -> Result<TokenPairsEntity, DbErr> {
        let active_model: crate::ports::db::model::token_pairs::ActiveModel = token_pair.to_model().into();
        let updated_model = TokenPairsRepository::update(active_model).await?;
        Ok(TokenPairsEntity::from_model(&updated_model))
    }
}