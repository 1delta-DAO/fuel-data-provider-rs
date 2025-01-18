#![feature(duration_constructors)]

use std::env;
use crate::config::CONFIG;
use crate::ports::blockchain::TxSync;
use crate::ports::db::database_manager::DB_MANAGER;
use crate::ports::tx_monitor_poc::TxMonitorPOC;

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
        log::info!("Starting TX Sync service ...");
        match TxSync::synchronize_transactions().await{
            Ok(_) => println!("Synchronization finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

/*    let tx_handle = tokio::spawn(async {
        log::info!("Starting TX monitoring service ...");
        match TxMonitorPOC::monitor_transactions().await {
            Ok(_) => println!("Monitoring finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });*/

    if let Err(e) = tokio::try_join!(tx_sync_handle) {
        log::error!("Error occurred while joining tasks: {:?}", e);
    }

    Ok(())
}
