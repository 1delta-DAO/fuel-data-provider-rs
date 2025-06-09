use crate::ports::db::model::unknown_token;
use crate::ports::db::model::unknown_token::Model;
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::DbErr;

pub struct UnknownTokenRepository;

#[async_trait]
impl CrudRepository<unknown_token::Entity> for UnknownTokenRepository {}

impl UnknownTokenRepository {
    /// Finds a token by its address
    pub async fn find_by_address(address: &str) -> Result<Option<Model>, DbErr> {
        Self::find_by_column(unknown_token::Column::Address, address.to_string()).await
    }
}
