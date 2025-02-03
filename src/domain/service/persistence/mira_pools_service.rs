use chrono::Utc;
use crate::ports::db::repository::{CrudRepository, MiraPoolsRepository};
use sea_orm::{ActiveValue, DbErr, IntoActiveModel};
use sea_orm::prelude::{DateTimeWithTimeZone, Decimal};
use uuid::Uuid;
use crate::domain::entity::entity::Entity;
use crate::domain::entity::mira_pools_entity::MiraPoolsEntity;
use crate::domain::entity::TokenEntity;

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

                println!("Pool not found, creating a new one...");
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

}