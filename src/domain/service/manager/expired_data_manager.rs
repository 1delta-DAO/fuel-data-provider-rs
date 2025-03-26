use std::time::Duration;
use crate::domain::service::persistence::{PairSwapsService, PriceDataService, VolumeDataService};
use crate::domain::service::exception::DataException;

pub struct ExpiredDataManager;

impl ExpiredDataManager {
    pub async fn cleanup_job() -> Result<(), DataException> {

        loop {
            match VolumeDataService::delete_expired().await {
                Ok(count) => {
                    if count > 0 {
                        log::info!("Deleted {} expired volume data records", count);
                    }
                },
                Err(err) => {
                    let error_msg = format!("Error deleting expired volume data: {:?}", err);
                    log::error!("{}", error_msg);
                    return Err(DataException::DataCleanupException(error_msg));
                }
            }

            match PairSwapsService::delete_expired().await {
                Ok(count) => {
                    if count > 0 {
                        log::info!("Deleted {} expired pair swaps records", count);
                    }
                },
                Err(err) => {
                    let error_msg = format!("Error deleting expired pair swaps data: {:?}", err);
                    log::error!("{}", error_msg);
                    return Err(DataException::DataCleanupException(error_msg));
                }
            }

            match PriceDataService::delete_expired().await {
                Ok(count) => {
                    if count > 0 {
                        log::info!("Deleted {} expired price data records", count);
                    }
                },
                Err(err) => {
                    let error_msg = format!("Error deleting expired price data: {:?}", err);
                    log::error!("{}", error_msg);
                    return Err(DataException::DataCleanupException(error_msg));
                }
            }


            // Sleep for 1 minute before the next cleanup
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}