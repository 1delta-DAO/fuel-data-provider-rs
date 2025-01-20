use crate::domain::entity::entity::Entity;
use crate::ports::db::model::pair_swaps::Model;
use chrono::{DateTime, Utc};
use sea_orm::prelude::Decimal;
use uuid::Uuid;

pub struct PairSwapsEntity {
    pub id: Uuid,
    pub block_number: String,
    pub block_time: Option<DateTime<Utc>>,
    pub tx_id: String,
    pub utxo_id: String,
    pub pair_id: Uuid,
    pub base_amount: Decimal,
    pub quote_amount: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Entity<Model> for PairSwapsEntity {
    fn from_model(model: &Model) -> Self {

        let block_time: Option<DateTime<Utc>> = model.block_time.map(|bt| bt.with_timezone(&Utc));

        Self {
            id: model.id,
            block_number: model.block_number.clone(),
            block_time,
            tx_id: model.tx_id.clone(),
            utxo_id: model.utxo_id.clone(),
            pair_id: model.pair_id,
            base_amount: model.base_amount,
            quote_amount: model.quote_amount,
            created_at: model.created_at.with_timezone(&Utc),
            updated_at: model.updated_at.with_timezone(&Utc),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            block_number: self.block_number.clone(),
            block_time: self.block_time.map(|dt| dt.into()),
            tx_id: self.tx_id.clone(),
            utxo_id: self.utxo_id.clone(),
            pair_id: self.pair_id,
            base_amount: self.base_amount,
            quote_amount: self.quote_amount,
            created_at: self.created_at.into(),
            updated_at: self.updated_at.into(),
        }
    }
}

impl Default for PairSwapsEntity {
    fn default() -> Self {
        Self {
            id: Uuid::nil(),
            block_number: String::new(),
            block_time: None,
            tx_id: String::new(),
            utxo_id: String::new(),
            pair_id: Uuid::new_v4(),
            base_amount: Decimal::ZERO,
            quote_amount: Decimal::ZERO,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}