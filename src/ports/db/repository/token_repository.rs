use crate::ports::db::model::token;
use crate::ports::db::model::token::Model;
use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sea_orm::{ColumnTrait, Condition, DbErr, EntityTrait, Order, QueryFilter, QueryOrder};

pub struct TokenRepository;

#[async_trait]
impl CrudRepository<token::Entity> for TokenRepository {}

impl TokenRepository {
    /// Finds a token by its address
    pub async fn find_by_address(address: &str) -> Result<Option<Model>, DbErr> {
        Self::find_by_column(token::Column::Address, address.to_string()).await
    }

    pub async fn find_by_created_between(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Model>, DbErr> {
        let db = &crate::ports::db::database_manager::DB_MANAGER
            .get_connection()
            .await
            .unwrap();
        token::Entity::find()
            .filter(token::Column::CreatedAt.between(start, end))
            .all(db)
            .await
    }

    /// Finds tokens by a list of addresses
    pub async fn find_by_addresses(addresses: Vec<String>) -> Result<Vec<Model>, DbErr> {
        if addresses.is_empty() {
            return Ok(vec![]);
        }

        let db = &crate::ports::db::database_manager::DB_MANAGER
            .get_connection()
            .await
            .unwrap();

        let condition = addresses.iter().fold(Condition::any(), |c, addr| {
            c.add(token::Column::Address.eq(addr.clone()))
        });

        token::Entity::find().filter(condition).all(db).await
    }

    /// Finds tokens sorted by price_change24 in ascending order (losers)
    pub async fn find_sorted_by_price_change_asc() -> Result<Vec<Model>, DbErr> {
        let db = &crate::ports::db::database_manager::DB_MANAGER
            .get_connection()
            .await
            .unwrap();

        token::Entity::find()
            .order_by(token::Column::PriceChange24, Order::Asc)
            .all(db)
            .await
    }

    /// Finds tokens sorted by price_change24 in descending order (gainers)
    pub async fn find_sorted_by_price_change_desc() -> Result<Vec<Model>, DbErr> {
        let db = &crate::ports::db::database_manager::DB_MANAGER
            .get_connection()
            .await
            .unwrap();

        token::Entity::find()
            .order_by(token::Column::PriceChange24, Order::Desc)
            .all(db)
            .await
    }

    /// Finds tokens sorted by volume24 in descending order (highest volume first)
    pub async fn find_sorted_by_volume_desc() -> Result<Vec<Model>, DbErr> {
        let db = &crate::ports::db::database_manager::DB_MANAGER
            .get_connection()
            .await
            .unwrap();

        token::Entity::find()
            .order_by(token::Column::Volume24Usd, Order::Desc)
            .all(db)
            .await
    }
}
