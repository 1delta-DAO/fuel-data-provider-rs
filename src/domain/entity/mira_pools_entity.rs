use crate::domain::entity::entity::Entity;
use crate::ports::db::model::mira_pools::Model;
use chrono::{DateTime, Utc};
use sea_orm::prelude::Decimal;
use uuid::Uuid;

pub struct MiraPoolsEntity {
    pub id: Uuid,
    pub pair_id: Uuid,
    pub swaps: i64,
    pub reserve_base: Decimal,
    pub reserve_quote: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity<Model> for MiraPoolsEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            pair_id: model.pair_id,
            swaps: model.swaps,
            reserve_base: model.reserve_base,
            reserve_quote: model.reserve_quote,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            pair_id: self.pair_id,
            swaps: self.swaps,
            reserve_base: self.reserve_base,
            reserve_quote: self.reserve_quote,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
        }
    }
}

impl Default for MiraPoolsEntity {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            pair_id: Uuid::new_v4(),
            swaps: 0,
            reserve_base: Decimal::ZERO,
            reserve_quote: Decimal::ZERO,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}