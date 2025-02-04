use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use num_traits::AsPrimitive;
use reqwest::Client;
use serde_json::json;
use crate::config::CONFIG;
use crate::ports::sentio::{ApiResponse, Pool, SwapEvent, SyncSqlResult};

const BATCH_SIZE: usize = 100;

pub struct SubgraphQueryService {
    client: Client,
    endpoint: String,
    api_key: String,
    cache: Arc<Mutex<HashMap<String, SwapEvent>>>,
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

    pub async fn initialize_cache(&self, block_start: u32, block_end: u32) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Initializing cache: {}-{}",block_start,block_end);
        let mut offset = 0;
        let mut last_block: String = "".to_string();

        loop {
            let request_body = json!({
                "block_number_start": block_start.to_string(),
                "block_number_end": block_end.to_string(),
                "batch_size": BATCH_SIZE.to_string(),
                "offset": offset.to_string()
            });

            let response = self.client
                .post(&self.endpoint)
                .header("Content-Type", "application/json")
                .header("api-key", &self.api_key)
                .json(&request_body)
                .send()
                .await?;

            let response_text = response.text().await?;
            //log::info!("Response: {}", response_text);
            let parsed_response: ApiResponse = serde_json::from_str(&response_text)
                .map_err(|e| {
                    log::error!("JSON conversion exception: {:?}", e);
                    e
                })?;
            //log::info!("Fetched rows: {}", parsed_response.sync_sql_response.result.rows.as_ref().map_or(0, |r| r.len()));

            let mut cache = self.cache.lock().unwrap();
            if let Some(rows) = parsed_response.sync_sql_response.result.rows {
                let batch_size = rows.len();
                for row in rows {
                    cache.insert(row.transaction_hash.clone(), row.clone());
                    last_block = row.clone().block_number.to_string();
                }
                log::info!("Cache size {}", cache.len());
                if batch_size < BATCH_SIZE {
                    break;
                }

                offset += BATCH_SIZE;
            } else {
                break;
            }
        }
        log::info!("Cache updated to block: {}", last_block);
        Ok(())
    }

    pub fn get_by_transaction_hash(&self, tx_hash: &str) -> Option<SwapEvent> {
        let cache = self.cache.lock().unwrap();
        cache.get(tx_hash).cloned()
    }

    pub fn get_logs_by_block_number_from_cache(&self, block_number: u32) -> Vec<SwapEvent> {
        let cache = self.cache.lock().unwrap();
        let results: Vec<SwapEvent> = cache
            .values()
            .filter(|event| event.block_number == block_number as u64) // 🔹 Filtrujemy po `block_number`
            .cloned()
            .collect();

        //log::info!("{} results found for block_number {}", results.len(), block_number);
        results
    }

    pub async fn get_logs_by_block_number(&self, block_number: u32) -> Result<Vec<SwapEvent>, Box<dyn std::error::Error>> {

        let request_body = serde_json::json!({
            "block_number_start": (block_number).to_string(),
            "block_number_end": block_number.to_string(),
            "batch_size": BATCH_SIZE.to_string(),
            "offset": "0".to_string()
        });

        let response = self.client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await?;
    
        let response_text = response.text().await?;
        let parsed_response: ApiResponse = serde_json::from_str(&response_text)
            .map_err(|e| {
                log::error!("JSON conversion exception : {:?}", e);
                e
            })?;
        log::info!("parsed_response: `{:?}`",parsed_response);
    
        let rows = parsed_response.sync_sql_response.result.rows.unwrap_or_default();
        Ok(rows)

    }
}