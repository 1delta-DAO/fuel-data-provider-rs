use crate::ports::db::repository::CrudRepository;
use async_trait::async_trait;
use crate::ports::db::model::volume_data;

pub struct VolumeDataRepository;

#[async_trait]
impl CrudRepository<volume_data::Entity> for VolumeDataRepository {}
