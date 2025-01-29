use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use reqwest::Client;
use serde_json::json;
use crate::config::CONFIG;
use crate::ports::sentio::{ApiResponse, Pool, SwapData};

pub struct SubgraphQueryService {
    client: Client,
    endpoint: String,
    api_key: String, // Klucz API
    cache: Arc<Mutex<HashMap<String, SwapData>>>, // Cache w pamięci
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

    async fn get_log_by_transaction_hash(
        &self,
        transaction_hash: &str,
    ) -> Result<Option<SwapData>, Box<dyn std::error::Error>> {
        let cache = self.cache.lock().unwrap();

        if let Some(log) = cache.get(transaction_hash) {
            return Ok(Some(log.clone()));
        }

        drop(cache);
        self.refresh_cache().await?;

        let cache = self.cache.lock().unwrap();
        Ok(cache.get(transaction_hash).cloned())
    }

    pub async fn get_all_logs(&self) -> Result<Vec<SwapData>, Box<dyn std::error::Error>> {
        let cache = self.cache.lock().unwrap();

        if !cache.is_empty() {
            return Ok(cache.values().cloned().collect());
        }

        drop(cache);
        self.refresh_cache().await?;

        let cache = self.cache.lock().unwrap();
        Ok(cache.values().cloned().collect())
    }

    async fn refresh_cache(&self) -> Result<(), Box<dyn std::error::Error>> {
        let query = json!({
            "query": r#"
                {
                    syncSqlResponse {
                        runtimeCost
                        result {
                            rows {
                                address
                                block_number
                                chain
                                contract
                                dex
                                distinct_event_id
                                event_name
                                log_index
                                poolId
                                recipient
                                timestamp
                                token0In
                                token0Out
                                token1In
                                token1Out
                                transaction_hash
                                transaction_index
                            }
                        }
                    }
                }
            "#
        });

        let response = self
            .client
            .post(&self.endpoint)
            .header("Content-Type", "application/json")
            .header("api-key", &self.api_key) // Dodano nagłówek z kluczem API
            .json(&query)
            .send()
            .await?
            .json::<ApiResponse>()
            .await?;

        let mut cache = self.cache.lock().unwrap();
        for row in response.syncSqlResponse.result.rows {
            cache.insert(row.transaction_hash.clone(), row);
        }

        Ok(())
    }
}