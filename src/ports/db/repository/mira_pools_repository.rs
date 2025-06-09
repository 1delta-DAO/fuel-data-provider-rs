use crate::ports::db::model::mira_pools::{self, Model};
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};

pub struct MiraPoolsRepository;

#[async_trait]
impl CrudRepository<mira_pools::Entity> for MiraPoolsRepository {}

impl MiraPoolsRepository {
    /// Finds a pool by `pair_id`
    pub async fn find_by_pair_id(pair_id: uuid::Uuid) -> Result<Option<Model>, DbErr> {
        mira_pools::Entity::find()
            .filter(mira_pools::Column::PairId.eq(pair_id))
            .one(&crate::ports::db::database_manager::DB_MANAGER.get_connection().await.unwrap())
            .await
    }
}
