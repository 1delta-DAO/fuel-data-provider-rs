use std::collections::HashMap;
use chrono::Utc;
use num_traits::ToPrimitive;
use crate::ports::db::repository::{CrudRepository, MiraPoolsRepository, TokenPairsRepository, TokenRepository};
use sea_orm::{ActiveValue, DbErr, IntoActiveModel};
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::mira_pools_entity::MiraPoolsEntity;
use crate::domain::entity::{TokenEntity, TokenPairsEntity};
use crate::domain::utils::Converter;
use crate::ports::db::model::token;

pub struct MiraPoolsService;

impl MiraPoolsService {
    /// Finds a pool by `pair_id`
    pub async fn find_by_pair_id(pair_id: uuid::Uuid) -> Result<Option<MiraPoolsEntity>, DbErr> {
        if let Some(model) = MiraPoolsRepository::find_by_pair_id(pair_id).await? {
            Ok(Some(MiraPoolsEntity::from_model(&model)))
        } else {
            Ok(None)
        }
    }

    /// Creates a new pool record
    pub async fn create(pool_entity: MiraPoolsEntity) -> Result<MiraPoolsEntity, DbErr> {
        let model = pool_entity.to_model();
        let created_model = MiraPoolsRepository::create(model.into_active_model()).await?;
        Ok(MiraPoolsEntity::from_model(&created_model))
    }

    /// Updates an existing pool record
    pub async fn update(pool_entity: MiraPoolsEntity) -> Result<MiraPoolsEntity, DbErr> {
        let mut active_model: crate::ports::db::model::mira_pools::ActiveModel = pool_entity.to_model().into();
        active_model.reserve_base = ActiveValue::Set(Decimal::from(pool_entity.swaps));
        active_model.reserve_base = ActiveValue::Set(pool_entity.reserve_base);
        active_model.reserve_quote = ActiveValue::Set(pool_entity.reserve_quote);
        active_model.updated_at = ActiveValue::Set(DateTimeWithTimeZone::from(chrono::Utc::now()));
        let updated_model = MiraPoolsRepository::update(active_model).await?;
        Ok(MiraPoolsEntity::from_model(&updated_model))
    }

    /// Finds an existing pool by `pair_id` or creates a new one
    pub async fn find_or_create(pair_id: Uuid) -> Result<MiraPoolsEntity, DbErr> {
        match Self::find_by_pair_id(pair_id).await? {
            Some(existing_pool) => {
                let updated_pool = Self::update(existing_pool).await?;
                Ok(updated_pool)
            },
            None => {
                let mira_pool = MiraPoolsEntity{
                    id: Uuid::new_v4(),
                    pair_id,
                    swaps: 0,
                    reserve_base: Decimal::ZERO,
                    reserve_quote: Decimal::ZERO,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                };

                log::info!("Pool not found, creating a new one...");
                match Self::create(mira_pool).await{
                    Ok(result)=>{
                        Ok(result)
                    },
                    Err(err)=>{
                        log::info!("Error during mira_pool creation: {}",err);
                        Err(err)
                    }
                }
            }
        }
    }

    pub async fn collect_token_liquidity() -> Result<HashMap<Uuid, Decimal>, DbErr> {
        // Get all pools
        let all_pools = MiraPoolsRepository::find_all().await?;
        let pools_entities: Vec<MiraPoolsEntity> = all_pools
            .iter()
            .map(|model| MiraPoolsEntity::from_model(model))
            .collect();

        // Get all token pairs to link pools with tokens
        let all_pairs = TokenPairsRepository::find_all().await?;
        let pairs_entities: Vec<TokenPairsEntity> = all_pairs
            .iter()
            .map(|model| TokenPairsEntity::from_model(model))
            .collect();

        // Create a map of pair_id -> token pair for faster lookups
        let mut pair_map: HashMap<Uuid, &TokenPairsEntity> = HashMap::new();
        for pair in &pairs_entities {
            pair_map.insert(pair.id, pair);
        }

        // Aggregate liquidity for each token
        let mut token_liquidity: HashMap<Uuid, Decimal> = HashMap::new();

        for pool in &pools_entities {
            // Find the token pair for this pool
            if let Some(pair) = pair_map.get(&pool.pair_id) {
                // Add liquidity for base token
                let base_token_id = pair.base_token_details_id;
                let base_liquidity_entry = token_liquidity.entry(base_token_id).or_insert(Decimal::ZERO);
                *base_liquidity_entry += pool.reserve_base.clone();

                // Add liquidity for quote token
                let quote_token_id = pair.quote_token_details_id;
                let quote_liquidity_entry = token_liquidity.entry(quote_token_id).or_insert(Decimal::ZERO);
                *quote_liquidity_entry += pool.reserve_quote.clone();
            }
        }

        Ok(token_liquidity)
    }

    pub async fn prepare_tokens_liquidity() -> Result<Vec<TokenEntity>, DbErr> {
        // Get aggregated liquidity for tokens
        let token_liquidity = Self::collect_token_liquidity().await?;

        // Get all tokens
        let all_tokens = TokenRepository::find_all().await?;

        // Prepare the result vector
        let mut prepared_tokens: Vec<TokenEntity> = Vec::new();

        // Prepare each token entity with the appropriate liquidity
        for token_model in all_tokens {
            let token_id = token_model.id;

            // If we have liquidity data for this token
            let raw_liquidity = token_liquidity.get(&token_id)
                .cloned()
                .unwrap_or(Decimal::ZERO);

            let divisor = 10.0_f64.powi(token_model.decimals);
            let liquidity = raw_liquidity.to_f64().unwrap_or(0.0) / divisor;


            // Convert to entity
            let mut token_entity = TokenEntity::from_model(&token_model);

            // Update entity with liquidity value (convert to f64)
            let liquidity_f64 = Converter::round_f64(liquidity.to_f64().unwrap_or(0.0), token_entity.decimals);
            token_entity.liquidity = liquidity_f64;

            // Add token entity to the result
            prepared_tokens.push(token_entity);

        }

        Ok(prepared_tokens)
    }

}