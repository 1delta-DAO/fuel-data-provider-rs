use std::time::Duration;
use crate::domain::service::exception::DataException;
use crate::domain::service::persistence::{PriceDataService, TokenService, VolumeDataService};
use crate::domain::utils::Converter;

pub struct CalculationManager;

impl CalculationManager {
    pub async fn calculate_stats_job() -> Result<(), DataException> {

        loop{
            log::info!("Calculating stats...");

            let tokens = TokenService::find_all_tokens().await.unwrap();
            for mut token in tokens {

                // volume24

                let mut total_volume: f64 = 0.0;

                let volume_data_result = VolumeDataService::find_by_token_id(&token.id).await.unwrap();
                if volume_data_result.is_some(){

                    let volume_data = volume_data_result.unwrap();
                    total_volume = volume_data.iter()
                        .map(|data| data.volume)
                        .sum();

                }

                log::info!(
                    "Token: {} - total volume: {:.2}",
                    token.symbol,
                    total_volume
                );

                token.volume_24 = Converter::round_f64(total_volume,token.decimals);
                TokenService::update_volume(token.clone()).await.unwrap();

                // price_change_24

                let token_opening_price = PriceDataService::find_oldest_by_token_id(&token.id).await.unwrap();
                if token_opening_price.is_some() {
                    let opening_price = Converter::round_f64(token_opening_price.unwrap().price,token.decimals);
                    let current_price = Converter::round_f64(token.price.clone(),token.decimals);
                    log::info!("Opening price: {}", opening_price);
                    log::info!("Current price: {}", current_price);
                    let price_change24 = (((current_price - opening_price) / opening_price) * 100.0) as f32;
                    token.price_change24 = Converter::round_f32(price_change24,2);
                    log::info!("Price change 24: {}", token.price_change24);
                }
                else {
                    token.price_change24 = 0.0;
                    token.price = 0.0;
                }

                TokenService::update_price_change(token.clone()).await.unwrap();
                log::info!("Price change 24: {:.2}", token.price_change24);

                //let price_data = PriceDataService::find_by_token_id(&token.id).await.unwrap();
            }

            // Sleep for 1 minute before calculations
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}