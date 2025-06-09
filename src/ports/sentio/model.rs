use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapEvent {
    timestamp: String,
    #[serde(rename = "token0In")]
    pub token_0in: String,
    #[serde(rename = "token0Out")]
    pub token_0out: String,
    #[serde(rename = "token1In")]
    pub token_1in: String,
    #[serde(rename = "token1Out")]
    pub token_1out: String,
    dex: String,
    #[serde(rename = "poolId")]
    pub pool_id: String,
    pub recipient: String,
    pub address: String,
    pub block_number: u64,
    chain: String,
    contract: String,
    pub transaction_hash: String,
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

        Some(Pool { token0_address, token1_address, flag })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncSqlResult {
    pub rows: Option<Vec<SwapEvent>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncSqlResponse {
    #[serde(rename = "runtimeCost")]
    pub runtime_cost: String,
    pub result: SyncSqlResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResponse {
    #[serde(rename = "syncSqlResponse")]
    pub sync_sql_response: SyncSqlResponse,
}
