use crate::domain::entity::entity::Entity;
use crate::ports::db::model::sync_status::Model;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct SyncStatusEntity {
    pub id: Uuid,
    pub block_number: i32,
    pub block_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity<Model> for SyncStatusEntity {
    fn from_model(model: &Model) -> Self {

        let block_time: Option<DateTime<Utc>> = model.block_time.map(|bt| bt.with_timezone(&Utc));

        Self {
            id: model.id,
            block_number: model.block_number,
            block_time,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            block_number: self.block_number,
            block_time: self.block_time.map(|dt| dt.into()),
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
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}
