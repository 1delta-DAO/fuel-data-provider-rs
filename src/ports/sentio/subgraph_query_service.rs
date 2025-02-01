use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use num_traits::AsPrimitive;
use reqwest::Client;
use serde_json::json;
use crate::config::CONFIG;
use crate::ports::sentio::{ApiResponse, Pool, SwapData};

const BATCH_SIZE: usize = 10000;

pub struct SubgraphQueryService {
    client: Client,
    endpoint: String,
    api_key: String,
    cache: Arc<Mutex<HashMap<String, SwapData>>>,
}

impl SubgraphQueryService {
    pub fn new() -> Self {
        SubgraphQueryService {
            client: Client::new(),
            endpoint: CONFIG.default.sentio_url.clone(),
            api_key: CONFIG.default.sentio_api_key.clone(),
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get_logs_by_block_number(&self, block_number: u32) -> Result<Vec<SwapData>, Box<dyn std::error::Error>> {


        let raw_data = format!(r#"{{ "block_number": "{}" }}"#, block_number.to_string());

        let response = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("api-key", &self.api_key)
            .body(raw_data)
            .send()
            .await?;

        if !response.status().is_success() {
            log::error!("API Error: {:?}", response.status());
            return Err(Box::from("No API in data available"));
        }

        let parsed_response = response.json::<ApiResponse>().await?;
        let rows = parsed_response.syncSqlResponse.result.rows;

        Ok(rows)

    }

    async fn get_log_by_transaction_hash(
        &self,
        transaction_hash: &str,
    ) -> Result<Option<SwapData>, Box<dyn std::error::Error>> {
        {
            let cache = self.cache.lock().unwrap();

            if let Some(log) = cache.get(transaction_hash) {
                return Ok(Some(log.clone()));
            }
        }


        //drop(cache);
        self.refresh_cache().await?;

        let cache = self.cache.lock().unwrap();
        Ok(cache.get(transaction_hash).cloned())
    }

    pub async fn get_all_logs(&self) -> Result<Vec<SwapData>, Box<dyn std::error::Error>> {
        {
            let cache = self.cache.lock().unwrap();
            if !cache.is_empty() {
                return Ok(cache.values().cloned().collect());
            }
        }

        self.refresh_cache().await?;

        let cache = self.cache.lock().unwrap();
        let result = cache.values().cloned().collect();

        //log::info!("Result: {:?}", result);
        Ok(result)
    }

    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut cache = self.cache.lock().unwrap();
        cache.clear();
        drop(cache);

        let mut skip = 0;
        let mut last_batch_size = BATCH_SIZE;

        let block_number = "13136230";

        while last_batch_size == BATCH_SIZE {
            let raw_data = format!(r#"{{ "block_number": "{}" }}"#, block_number);

            let response = self
                .client
                .post(&self.endpoint)
                .header("Content-Type", "application/json")
                .header("api-key", &self.api_key)
                .body(raw_data)
                .send()
                .await?;
            log::info!("Response: {:?}", response);

            if !response.status().is_success() {
                log::error!("API Error: {:?}", response.status());
                return Err(Box::from("No API in data available"));
            }

            let parsed_response = response.json::<ApiResponse>().await?;
            log::info!("Parsed response: {:?}", parsed_response);
            let rows = parsed_response.syncSqlResponse.result.rows;
            last_batch_size = rows.len();

            if rows.is_empty() {
                log::info!("Pagination end.");
                break;
            }

            let mut cache = self.cache.lock().unwrap();
            let initial_size = cache.len();

            for row in rows {
                if(cache.contains_key(&row.transaction_hash)) {
                    //log::warn!("Record duplicated: {:?}", row);
                }
                cache.insert(row.transaction_hash.clone(), row);
            }

            log::info!(
                "Retrived {} new records, totally in cache: {} (skip={})",
                last_batch_size,
                cache.len(),
                skip
            );

            skip += BATCH_SIZE;
        }

        log::info!("Records in cache: {}", self.cache.lock().unwrap().len());
        Ok(())
    }
}