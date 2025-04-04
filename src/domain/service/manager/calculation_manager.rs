use std::time::Duration;
use crate::config::CONFIG;
use crate::domain::service::exception::DataException;
use crate::domain::service::persistence::{PriceDataService, TokenService, VolumeDataService};
use crate::domain::service::persistence::mira_pools_service::MiraPoolsService;
use crate::domain::utils::Converter;

pub struct CalculationManager;

impl CalculationManager {
    pub async fn calculate_stats_job() -> Result<(), DataException> {

        loop{
            log::info!("Calculating stats...");

            let tokens = TokenService::find_all_tokens().await.unwrap();
            let liquidity = MiraPoolsService::prepare_tokens_liquidity().await.unwrap();
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
                token.volume_24_usd = Converter::round_f64(total_volume * token.price,token.decimals);
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
                    //token.price = 0.0; - this will delete price if there is no updates in the last 24h
                }

                TokenService::update_price_change(token.clone()).await.unwrap();
                log::info!("Price change 24: {:.2}", token.price_change24);

                let found_token = liquidity.iter().find(|token_liquidity| token_liquidity.address == token.address);
                match found_token {
                    Some(token_liquidity) => {
                        token.liquidity = token_liquidity.liquidity;
                    },
                    None => {
                        token.liquidity = 0.0;
                    }
                }
                token.no_liquidity = false;
                if token.quoting{
                    token.liquidity_usd = Converter::round_f64(token.liquidity,token.decimals);
                }else{
                    token.liquidity_usd = Converter::round_f64(token.liquidity * token.price,token.decimals);
                }
                if token.liquidity_usd == 0.0 {
                    token.no_liquidity = true;
                }

                let prices = PriceDataService::find_all_by_token_id(&token.id).await.unwrap();

                token.high_risk=false;
                if prices.len() < CONFIG.default.high_risk_swaps.into() || token.liquidity_usd < CONFIG.default.high_risk_liquidity.into() {
                    token.high_risk=true;
                }

                TokenService::update_liquidity(token.clone()).await.unwrap();


                log::info!("Liquidity: {} | USD:{} ", token.liquidity, token.liquidity_usd);
            }

            // Sleep for 1 minute before calculations
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
}