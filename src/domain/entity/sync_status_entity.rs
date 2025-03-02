use crate::domain::entity::entity::Entity;
use crate::ports::db::model::sync_status::Model;
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug,Serialize)]
pub struct SyncStatusEntity {
    pub id: Uuid,
    pub block_number: i32,
    pub block_time: Option<DateTime<Utc>>,
    pub first_calculation_point: DateTime<Utc>,
    pub calculation_data_ready: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity<Model> for SyncStatusEntity {
    fn from_model(model: &Model) -> Self {

        let block_time: Option<DateTime<Utc>> = model.block_time.map(|bt| bt.with_timezone(&Utc));

        Self {
            id: model.id,
            block_number: model.block_number.clone(),
            block_time,
            first_calculation_point: model.first_calculation_point.with_timezone(&Utc),
            calculation_data_ready: model.calculation_data_ready.clone(),
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            block_number: self.block_number.clone(),
            block_time: self.block_time.map(|dt| dt.into()),
            first_calculation_point: self.first_calculation_point.into(),
            calculation_data_ready: self.calculation_data_ready.clone(),
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
        }
    }
}

impl Default for SyncStatusEntity {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            block_number: 0,
            block_time: None,
            first_calculation_point: Utc::now(),
            calculation_data_ready:false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
