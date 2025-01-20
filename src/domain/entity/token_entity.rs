use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token::Model;
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug)]
pub struct TokenEntity {
    pub id: Uuid,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity<Model> for TokenEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            address: model.address.clone(),
            symbol: model.symbol.clone(),
            name: model.name.clone(),
            decimals: model.decimals,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            address: self.address.clone(),
            symbol: self.symbol.clone(),
            name: self.name.clone(),
            decimals: self.decimals,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
            high_risk: false, // Defaulting to `false` as this field is omitted
            no_liquidity: false, // Defaulting to `false` as this field is omitted
        }
    }
}

impl Default for TokenEntity {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            address: String::new(),
            symbol: String::new(),
            name: String::new(),
            decimals: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}