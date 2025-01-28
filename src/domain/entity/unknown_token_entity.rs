use crate::domain::entity::entity::Entity;
use crate::ports::db::model::unknown_token::Model;
use uuid::Uuid;

#[derive(Debug)]
pub struct UnknownTokenEntity {
    pub id: Uuid,
    pub address: String,
}

impl Entity<Model> for UnknownTokenEntity {
    fn from_model(model: &Model) -> Self {
        Self {
            id: model.id,
            address: model.address.clone(),
        }
    }

    fn to_model(&self) -> Model {
        Model {
            id: self.id,
            address: self.address.clone(),
        }
    }
}

impl Default for UnknownTokenEntity {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            address: String::new(),
        }
    }
}
