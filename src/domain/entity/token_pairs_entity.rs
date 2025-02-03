use crate::domain::entity::entity::Entity;
use crate::ports::db::model::token_pairs::Model;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct TokenPairsEntity {
    pub id: Uuid,
    pub base_token_details_id: Uuid,
    pub quote_token_details_id: Uuid,
    pub base_address: String,
    pub quote_address: String,
    pub pair_address: String,
    pub base_symbol: String,
    pub quote_symbol: String,
}

impl Entity<Model> for TokenPairsEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            base_token_details_id: model.base_token_details_id,
            quote_token_details_id: model.quote_token_details_id,
            base_address: model.base_address.clone(),
            quote_address: model.quote_address.clone(),
            pair_address: model.pair_address.clone(),
            base_symbol: model.base_symbol.clone(),
            quote_symbol: model.quote_symbol.clone(),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            base_token_details_id: self.base_token_details_id,
            quote_token_details_id: self.quote_token_details_id,
            base_address: self.base_address.clone(),
            quote_address: self.quote_address.clone(),
            pair_address: self.pair_address.clone(),
            base_symbol: self.base_symbol.clone(),
            quote_symbol: self.quote_symbol.clone(),
        }
    }
}

impl Default for TokenPairsEntity {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            base_token_details_id: Uuid::new_v4(),
            quote_token_details_id: Uuid::new_v4(),
            base_address: String::new(),
            quote_address: String::new(),
            pair_address: String::new(),
            base_symbol: String::new(),
            quote_symbol: String::new(),
        }
    }
}