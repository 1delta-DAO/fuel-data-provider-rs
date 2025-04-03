use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token::Model;
use chrono::{DateTime, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use sea_orm::prelude::Decimal;
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug,Serialize, Clone)]
pub struct TokenEntity {
    pub id: Uuid,
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub price: f64,
    pub volume_24: f64,
    pub price_change24: f32,
    pub decimals: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub quoting: bool,
}

impl Entity<Model> for TokenEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            address: model.address.clone(),
            symbol: model.symbol.clone(),
            name: model.name.clone(),
            price: model.price.to_f64().unwrap(),
            volume_24: model.volume24.to_f64().unwrap(),
            price_change24: model.price_change24.to_f32().unwrap(),
            decimals: model.decimals.clone(),
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
            quoting: model.quoting.clone(),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            address: self.address.clone(),
            symbol: self.symbol.clone(),
            name: self.name.clone(),
            price: Decimal::from_f64(self.price.clone()).unwrap_or(Decimal::ZERO),
            volume24: Decimal::from_f64(self.volume_24.clone()).unwrap_or(Decimal::ZERO),
            price_change24: Decimal::from_f32(self.price_change24.clone()).unwrap_or(Decimal::ZERO),
            decimals: self.decimals.clone(),
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
            high_risk: false, // Defaulting to `false` as this field is omitted
            no_liquidity: false, // Defaulting to `false` as this field is omitted
            quoting: self.quoting.clone(),
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
            price: 0.0,
            volume_24: 0.0,
            price_change24: 0.0,
            decimals: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            quoting: false,
        }
    }
}