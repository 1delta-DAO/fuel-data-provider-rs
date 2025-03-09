#![feature(duration_constructors)]

use std::env;
use crate::config::CONFIG;
use crate::domain::service::cleanup::expired_data_manager::ExpiredDataManager;
use crate::ports::blockchain::TxSync;
use crate::ports::db::database_manager::DB_MANAGER;

mod ports;
mod domain;
mod api;
mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    log::info!("Starting application...");

    log::info!("Config: {}", CONFIG.default.server_port_http);
    let _ = DB_MANAGER.initialize().await;

    let tx_sync_handle = tokio::spawn(async{
        log::info!("Starting TX Sync service - Runner 1 ...");
        match TxSync::synchronize_transactions(1).await{
            Ok(_) => println!("Synchronization finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    let data_cleanup_handle = tokio::spawn(async {
       log::info!("Starting data cleanup job ...");
        match ExpiredDataManager::cleanup_job().await {
            Ok(_) => println!("Cleanup job finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    let server_handle = tokio::spawn(async {
        let routes = api::rest::routes::routes();
        log::info!("Starting HTTP server ...");
        warp::serve(routes).run(([0, 0, 0, 0], CONFIG.default.server_port_http.clone())).await;
    });


    if let Err(e) = tokio::try_join!(
        tx_sync_handle,
        server_handle,
        data_cleanup_handle
        ) {
        log::error!("Error occurred while joining tasks: {:?}", e);
    }

    Ok(())
}
