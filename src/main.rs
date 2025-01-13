use std::env;
use crate::config::CONFIG;
use crate::ports::tx_monitor::TxMonitor;

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

    let tx_handle = tokio::spawn(async {
        log::info!("Starting TX monitoring service ...");
        match TxMonitor::monitor_transactions().await {
            Ok(_) => println!("Monitoring finished successfully."),
            Err(e) => eprintln!("Error occurred: {}", e),
        }
    });

    if let Err(e) = tokio::try_join!(tx_handle) {
        log::error!("Error occurred while joining tasks: {:?}", e);
    }

    Ok(())
}
