use crate::ports::db::database_manager::DB_MANAGER;
use async_trait::async_trait;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, IntoActiveModel, PrimaryKeyTrait, QueryFilter, TransactionTrait};

#[allow(dead_code)]
#[async_trait]
pub trait CrudRepository<E>
where
    E: EntityTrait + Sync + Send + 'static,
    E::Model: IntoActiveModel<E::ActiveModel> + Sync + Send,
    E::ActiveModel: Sync + Send,
    E::PrimaryKey: Sync + Send,
    <E::PrimaryKey as PrimaryKeyTrait>::ValueType: Sync + Send,
{
    async fn find_by_id(id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType) -> Result<Option<E::Model>, sea_orm::DbErr> {
        E::find_by_id(id).one(&DB_MANAGER.get_connection().await.unwrap()).await
    }

    async fn find_all() -> Result<Vec<E::Model>, sea_orm::DbErr> {
        E::find().all(&DB_MANAGER.get_connection().await.unwrap()).await
    }

    async fn create(active_model: E::ActiveModel) -> Result<E::Model, sea_orm::DbErr> {
        active_model.insert(&DB_MANAGER.get_connection().await.unwrap()).await
    }

    async fn create_many(active_models: Vec<E::ActiveModel>) -> Result<(), sea_orm::DbErr> {
        if active_models.is_empty() {
            return Ok(());
        }

        let db = &DB_MANAGER.get_connection().await.unwrap();
        let txn = db.begin().await?;

        match E::insert_many(active_models).exec(&txn).await {
            Ok(_) => {
                txn.commit().await?;
                Ok(())
            }
            Err(e) => {
                txn.rollback().await?;
                Err(e)
            }
        }
    }

    async fn update(active_model: E::ActiveModel) -> Result<E::Model, sea_orm::DbErr> {
        log::info!("UPDATE {:?}",active_model);
        active_model.update(&DB_MANAGER.get_connection().await.unwrap()).await
    }

    async fn delete(id: <E::PrimaryKey as PrimaryKeyTrait>::ValueType) -> Result<(), sea_orm::DbErr> {
        E::delete_by_id(id).exec(&DB_MANAGER.get_connection().await.unwrap()).await.map(|_| ())
    }

    async fn find_by_column<C, V>(column: C, value: V) -> Result<Option<E::Model>, sea_orm::DbErr>
    where
        C: ColumnTrait + 'static,
        V: Into<sea_orm::Value> + Send,
    {
        E::find().filter(column.eq(value.into())).one(&DB_MANAGER.get_connection().await.unwrap()).await
    }
}
