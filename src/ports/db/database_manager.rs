use crate::config::CONFIG;
use once_cell::sync::Lazy;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct DatabaseManager {
    pool: Arc<Mutex<Option<DatabaseConnection>>>,
}

pub static DB_MANAGER: Lazy<DatabaseManager> = Lazy::new(|| DatabaseManager { pool: Arc::new(Mutex::new(None)) });

impl DatabaseManager {
    pub async fn initialize(&self) -> Result<(), String> {
        log::info!("DB connection initializing ...");
        let connection_url = format!("postgres://{}:{}@{}", CONFIG.default.db_username, CONFIG.default.db_password, CONFIG.default.db_url);
        let mut options = ConnectOptions::new(connection_url);
        options
            .max_connections(CONFIG.default.db_max_connections)
            .min_connections(CONFIG.default.db_min_connections)
            .sqlx_logging(CONFIG.default.db_sql_logging);

        match Database::connect(options).await {
            Ok(connection) => {
                let mut pool = self.pool.lock().await;
                *pool = Some(connection);
                log::info!("DB Connection pool created ...");
                Ok(())
            }
            Err(err) => Err(format!("Failed to initialize database pool: {:?}", err)),
        }
    }

    pub async fn get_connection(&self) -> Result<DatabaseConnection, String> {
        let pool = self.pool.lock().await;
        if let Some(connection) = pool.as_ref() {
            Ok(connection.clone())
        } else {
            Err("Database connection pool is not initialized".to_string())
        }
    }
}
