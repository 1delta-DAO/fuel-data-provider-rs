use crate::domain::entity::entity::Entity;
use crate::ports::db::model::volume_data::Model;
use chrono::{DateTime, Utc};
use num_traits::{FromPrimitive, ToPrimitive};
use sea_orm::prelude::Decimal;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct VolumeDataEntity {
    pub timestamp: DateTime<Utc>,
    pub token_id: Uuid,
    pub volume: f64,
}

impl Entity<Model> for VolumeDataEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            timestamp: model.timestamp.with_timezone(&Utc),
            token_id: model.token_id,
            volume: model.volume.to_f64().unwrap(),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            timestamp: self.timestamp.into(),
            token_id: self.token_id,
            volume: Decimal::from_f64(self.volume.clone()).unwrap(),
        }
    }
}

impl Default for VolumeDataEntity {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            token_id: Uuid::new_v4(),
            volume: 0.0,
        }
    }
}