#![feature(duration_constructors)]

use std::env;
use crate::config::CONFIG;
use crate::domain::service::manager::{CalculationManager, ExpiredDataManager};
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

    //
    // let missing_tokens = vec![
    //     "1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82",
    //     "91b3559edb2619cde8ffb2aa7b3c3be97efd794ea46700db7092abeee62281b0",
    //     "899e8a1f3326c80be9853977f43aa0579f7d4d3ed072643182402abf593b755d",
    //     "9faba055d60f126d67b7ea997a052d8fa27a422b6727f6f34603b13b889b8322",
    //     "2afcfa14b2eb778f08a0cb00995ee91a72e172e005596481e79bcab8b465ee78",
    //     "71ebe5293dfd31de8b62369487ebf1ed55e645043d54df91b8be596c02fa44f9",
    //     "a917ed563e47fe60994f758560e8b3b5876c3d6bab73149ce9412d9d5ba91988"
    // ];
    //
    // let sync = TxSync::new();
    //
    // let provider = Provider::connect(CONFIG.default.rpc_url_one.as_str()).await?;
    //
    // for address in missing_tokens{
    //     log::info!("Address: {}", address);
    //     let asset = AssetId::from_str(address).unwrap();
    //     get_mira_token_details_by_asset_id(&provider,&asset).await?;
    //     log::info!("Address: {} - OK", address);
    // }

    let tx_sync_handle = tokio::spawn(async{
        let mut retry_count = 0;
        loop{
            log::info!("Starting TX Sync service ...");
            match TxSync::synchronize_transactions(1).await{
                Ok(_) => {
                    retry_count = 0;
                    println!("Synchronization finished successfully.");
                },
                Err(e) => {
                    retry_count += 1;
                    eprintln!("Top level - Error occurred: {}", e);
                    println!("Retrying sync service job in {} seconds", calculate_backoff(retry_count));
                },
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(calculate_backoff(retry_count))).await;
        }
    });

    let data_cleanup_handle = tokio::spawn(async {
        let mut retry_count = 0;
        loop{
            log::info!("Starting data cleanup job ...");
            match ExpiredDataManager::cleanup_job().await {
                Ok(_) => {
                    retry_count = 0;
                    println!("Cleanup job finished successfully.");
                },
                Err(e) => {
                    retry_count += 1;
                    eprintln!("Top level - Error occurred: {}", e);
                    println!("Retrying data cleanup job in {} seconds", calculate_backoff(retry_count));
                },
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(calculate_backoff(retry_count))).await;
        }
    });

    let stats_calculation_handle = tokio::spawn(async {
        let mut retry_count = 0;
        loop{
            log::info!("Starting stats calculation job ...");
            match CalculationManager::calculate_stats_job().await {
                Ok(_) => {
                    retry_count = 0;
                    println!("Calculation job finished successfully.");
                },
                Err(e) => {
                    retry_count += 1;
                    eprintln!("Top level - Error occurred: {}", e);
                    println!("Retrying stats calculation job in {} seconds", calculate_backoff(retry_count));
                },
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(calculate_backoff(retry_count))).await;
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
        data_cleanup_handle,
        stats_calculation_handle,
        ) {
        log::error!("Error occurred while joining tasks: {:?}", e);
    }

    Ok(())
}

fn calculate_backoff(retry: u32) -> u64 {
    std::cmp::min(
        5 * 60, // 5 minutes
        2u64.saturating_pow(retry) * 5 // 5, 10, 20, 40, 80, 160
    )
}
