use std::time::Duration;
use crate::domain::service::exception::DataException;
use crate::domain::service::persistence::{PriceDataService, TokenService, VolumeDataService};

pub struct CalculationManager;

impl CalculationManager {
    pub async fn calculate_stats_job() -> Result<(), DataException> {

        loop{
            log::info!("Calculating stats...");

            let tokens = TokenService::find_all_tokens().await.unwrap();
            for mut token in tokens {
                let volume_data = VolumeDataService::find_by_token_id(&token.id).await.unwrap();
                let total_volume: f64 = volume_data.iter()
                    .map(|data| data.volume)
                    .sum();

                log::info!(
                    "Token: {} - volume records: {}, total volume: {:.2}",
                    token.symbol,
                    volume_data.len(),
                    total_volume
                );

                token.volume_24 = total_volume;
                TokenService::update_volume(token).await.unwrap();

                //let price_data = PriceDataService::find_by_token_id(&token.id).await.unwrap();
            }

            // Sleep for 1 minute before calculations
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}