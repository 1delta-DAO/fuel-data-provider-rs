use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use crate::ports::db::model::price_data;

pub struct PriceDataRepository;

#[async_trait]
impl CrudRepository<price_data::Entity> for PriceDataRepository {}
