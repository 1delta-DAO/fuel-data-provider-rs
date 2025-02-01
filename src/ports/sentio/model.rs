use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapData {
    pub address: String,
    pub block_number: u64,
    pub chain: String,
    pub contract: String,
    pub dex: String,
    pub distinct_event_id: String,
    pub event_name: String,
    pub log_index: u64,
    pub pool_id: String,
    pub recipient: String,
    pub timestamp: String,
    pub token0In: String,
    pub token0Out: String,
    pub token1In: String,
    pub token1Out: String,
    pub transaction_hash: String,
    pub transaction_index: u64,
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub token0_address: String,
    pub token1_address: String,
    pub flag: bool,
}

impl Pool {
    pub fn from_pool_id(pool_id: &str) -> Option<Self> {
        let parts: Vec<&str> = pool_id.split('-').collect();
        if parts.len() != 3 {
            return None;
        }

        let token0_address = parts[0].to_string();
        let token1_address = parts[1].to_string();
        let flag = parts[2].parse::<bool>().ok()?;

        Some(Pool {
            token0_address,
            token1_address,
            flag,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncSqlResult {
    pub rows: Vec<SwapData>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncSqlResponse {
    pub runtime_cost: String,
    pub result: SyncSqlResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResponse {
    pub sync_sql_response: SyncSqlResponse,
}