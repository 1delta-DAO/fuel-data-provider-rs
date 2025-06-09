use crate::config::CONFIG;
use crate::ports::db::database_manager::DB_MANAGER;
use crate::ports::db::model::volume_data;
use crate::ports::db::model::volume_data::Model;
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::ColumnTrait;
use sea_orm::DbErr;
use uuid::Uuid;

pub struct VolumeDataRepository;

#[async_trait]
impl CrudRepository<volume_data::Entity> for VolumeDataRepository {}

impl VolumeDataRepository {
    /// Finds volume records by id
    pub async fn find_by_token_id(token_id: &Uuid) -> Result<Option<Vec<Model>>, DbErr> {
        let result =
            Self::find_by_column_many(volume_data::Column::TokenId, token_id.to_owned()).await?;

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    pub async fn find_by_timestamp_and_token_id(
        timestamp: &chrono::DateTime<chrono::Utc>,
        token_id: &Uuid,
    ) -> Result<Option<Model>, DbErr> {
        use sea_orm::{Condition, EntityTrait, QueryFilter};

        let condition = Condition::all()
            .add(volume_data::Column::Timestamp.eq(timestamp.to_owned()))
            .add(volume_data::Column::TokenId.eq(token_id.to_owned()));

        volume_data::Entity::find()
            .filter(condition)
            .one(&DB_MANAGER.get_connection().await.unwrap())
            .await
    }

    /// Deletes volume data records older than the specified number of minutes
    pub async fn delete_expired() -> Result<u64, DbErr> {
        use chrono::{Duration, Utc};
        use sea_orm::{prelude::*, Condition};

        let minutes = CONFIG.default.calculation_window as i64;

        // Calculate the cutoff timestamp (current time minus specified minutes)
        let cutoff_time = Utc::now() - Duration::minutes(minutes);

        // Create the filter condition for records older than the cutoff time
        let condition = Condition::all().add(volume_data::Column::Timestamp.lt(cutoff_time));

        // Execute the delete operation
        let delete_result = volume_data::Entity::delete_many()
            .filter(condition)
            .exec(&DB_MANAGER.get_connection().await.unwrap())
            .await?;

        // Return the number of rows affected
        Ok(delete_result.rows_affected)
    }
}
