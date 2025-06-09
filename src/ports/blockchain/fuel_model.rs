use crate::ports::blockchain::tx_sync::SwapEvent;

#[derive(Debug, Clone)]
pub struct Swap {
    pub tx_id: String,
    pub swap_event: SwapEvent,
}

#[derive(Debug, Clone)]
pub struct Pool {
    pub token0_address: String,
    pub token1_address: String,
}

impl Pool {
    pub fn from_swap(swap: &SwapEvent) -> Option<Self> {
        let (asset1, asset2, _active) = swap.pool_id;

        Some(Pool { token0_address: asset1.to_string(), token1_address: asset2.to_string() })
    }
}
