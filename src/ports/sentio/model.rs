use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SwapEvent {
    timestamp: String,
    token0In: String,
    token0Out: String,
    token1In: String,
    token1Out: String,
    dex: String,
    poolId: String,
    recipient: String,
    address: String,
    block_number: u64,
    chain: String,
    contract: String,
    transaction_hash: String,
}

impl SwapEvent {
    pub fn transaction_hash(&self) -> &str {
        &self.transaction_hash
    }

    pub fn token0In(&self) -> &str {
        &self.token0In
    }

    pub fn token1In(&self) -> &str {
        &self.token1In
    }

    pub fn token0Out(&self) -> &str {
        &self.token0Out
    }

    pub fn token1Out(&self) -> &str {
        &self.token1Out
    }

    pub fn poolId(&self) -> &str {
        &self.poolId
    }
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
    pub rows: Vec<SwapEvent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SyncSqlResponse {
    pub runtimeCost: String,
    pub result: SyncSqlResult,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ApiResponse {
    pub syncSqlResponse: SyncSqlResponse,
}
