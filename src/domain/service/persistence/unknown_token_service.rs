use crate::domain::entity::entity::Entity;
use crate::domain::entity::UnknownTokenEntity;
use crate::ports::db::model::unknown_token::{self};
use crate::ports::db::repository::{CrudRepository, UnknownTokenRepository};
use sea_orm::{DbErr, IntoActiveModel};

pub struct UnknownTokenService;

#[allow(dead_code)]
impl UnknownTokenService {
    /// Finds a token by its address
    pub async fn find_by_address(address: &str) -> Result<Option<UnknownTokenEntity>, DbErr> {
        if let Some(model) = UnknownTokenRepository::find_by_address(address).await? {
            Ok(Some(UnknownTokenEntity::from_model(&model)))
        } else {
            Ok(None)
        }
    }

    /// Creates a new token
    pub async fn create(token_entity: UnknownTokenEntity) -> Result<UnknownTokenEntity, DbErr> {
        let model = token_entity.to_model();
        let created_model = UnknownTokenRepository::create(model.into_active_model()).await?;
        Ok(UnknownTokenEntity::from_model(&created_model))
    }

    /// Creates a new token only if it does not exist
    pub async fn create_if_not_exists(
        token_entity: UnknownTokenEntity,
    ) -> Result<Option<UnknownTokenEntity>, DbErr> {
        if Self::find_by_address(&token_entity.address).await?.is_none() {
            Ok(Some(Self::create(token_entity).await?))
        } else {
            log::info!("Token with address {} already exists", token_entity.address);
            Ok(None)
        }
    }

    /// Updates an existing token
    pub async fn update(token_entity: UnknownTokenEntity) -> Result<UnknownTokenEntity, DbErr> {
        let active_model: unknown_token::ActiveModel = token_entity.to_model().into();
        let updated_model = UnknownTokenRepository::update(active_model).await?;
        Ok(UnknownTokenEntity::from_model(&updated_model))
    }
}
