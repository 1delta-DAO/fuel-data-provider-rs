use crate::ports::tx_monitor::TxMonitor;

mod ports;
mod domain;
mod api;
mod constants;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    env_logger::init();

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
