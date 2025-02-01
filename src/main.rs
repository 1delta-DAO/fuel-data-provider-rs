#![feature(duration_constructors)]

use std::env;
use crate::config::CONFIG;
use crate::ports::blockchain::{BlockchainDataService, TxSync};
use crate::ports::db::database_manager::DB_MANAGER;
use crate::ports::sentio::SubgraphQueryService;

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

/*    let service = SubgraphQueryService::new();
    let logs = service.get_logs_by_block_number(13231432).await;
    for log in logs {
        log::info!("LOG - len: {}", log.len());
        for row in log{
            log::info!("ROW: {:?}", row);
//            BlockchainDataService::get_block_time(row.block_number)
        }
    }*/

    log::info!("Starting application...");
    log::info!("Config: {}", CONFIG.default.server_port_http);
    let _ = DB_MANAGER.initialize().await;

    let tx_sync_handle_one = tokio::spawn(async{
        log::info!("Starting TX Sync service - Runner 1 ...");
        match TxSync::synchronize_transactions(1).await{
            Ok(_) => println!("Synchronization finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    let tx_sync_handle_two = tokio::spawn(async{
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        log::info!("Starting TX Sync service - Runner 2 ...");
        match TxSync::synchronize_transactions(2).await{
            Ok(_) => println!("Synchronization finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    let tx_sync_handle_three = tokio::spawn(async{
        tokio::time::sleep(std::time::Duration::from_secs(20)).await;
        log::info!("Starting TX Sync service - Runner 3 ...");
        match TxSync::synchronize_transactions(3).await{
            Ok(_) => println!("Synchronization finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    let tx_sync_handle_four = tokio::spawn(async{
        tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        log::info!("Starting TX Sync service - Runner 4 ...");
        match TxSync::synchronize_transactions(4).await{
            Ok(_) => println!("Synchronization finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    let tx_sync_handle_five = tokio::spawn(async{
        tokio::time::sleep(std::time::Duration::from_secs(40)).await;
        log::info!("Starting TX Sync service - Runner 5 ...");
        match TxSync::synchronize_transactions(5).await{
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

    if let Err(e) = tokio::try_join!(
        tx_sync_handle_one
/*        ,
        tx_sync_handle_two,
        tx_sync_handle_three,
        tx_sync_handle_four,
        tx_sync_handle_five*/
        ) {
        log::error!("Error occurred while joining tasks: {:?}", e);
    }

    Ok(())
}
