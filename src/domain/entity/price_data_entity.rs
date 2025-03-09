use crate::domain::entity::entity::Entity;
use crate::ports::db::model::price_data::Model;
use chrono::{DateTime, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use sea_orm::prelude::Decimal;
use uuid::Uuid;

pub struct PriceDataEntity {
    pub id: Uuid,
    pub token_id: Uuid,
    pub price: f64,
    pub timestamp: DateTime<Utc>,
}

impl Entity<Model> for PriceDataEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            token_id: model.token_id,
            price: model.price.to_f64().unwrap(),
            timestamp: model.timestamp.with_timezone(&Utc),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            token_id: self.token_id,
            price: Decimal::from_f64(self.price.clone()).unwrap(),
            timestamp: self.timestamp.into(),
        }
    }
}

impl Default for PriceDataEntity {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            token_id: Uuid::new_v4(),
            price: 0.0,
            timestamp: Utc::now(),
        }
    }
}