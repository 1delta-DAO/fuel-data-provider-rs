use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use sea_orm::DbErr;
use uuid::Uuid;
use crate::config::CONFIG;
use crate::ports::db::database_manager::DB_MANAGER;
use crate::ports::db::model::price_data;
use crate::ports::db::model::price_data::Model;
use sea_orm::{Condition, prelude::*};
use chrono::{Utc, Duration};

pub struct PriceDataRepository;

#[async_trait]
impl CrudRepository<price_data::Entity> for PriceDataRepository {}
impl PriceDataRepository {

    pub async fn find_oldest_by_token_id(token_id: &Uuid) -> Result<Option<Model>, DbErr> {
        use sea_orm::{EntityTrait, QueryFilter, QueryOrder, ColumnTrait};

        price_data::Entity::find()
            .filter(price_data::Column::TokenId.eq(token_id.to_owned()))
            .order_by_asc(price_data::Column::Timestamp)
            .one(&DB_MANAGER.get_connection().await.unwrap())
            .await
    }

    /// Deletes price data records older than the specified number of minutes
    pub async fn delete_expired() -> Result<u64, DbErr> {

        let minutes = CONFIG.default.calculation_window as i64;

        // Calculate the cutoff timestamp (current time minus specified minutes)
        let cutoff_time = Utc::now() - Duration::minutes(minutes);

        // Create the filter condition for records older than the cutoff time
        let condition = Condition::all().add(price_data::Column::Timestamp.lt(cutoff_time));

        // Execute the delete operation
        let delete_result = price_data::Entity::delete_many()
            .filter(condition)
            .exec(&DB_MANAGER.get_connection().await.unwrap())
            .await?;

        // Return the number of rows affected
        Ok(delete_result.rows_affected)
    }

}