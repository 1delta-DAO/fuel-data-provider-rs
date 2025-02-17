use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use fuel_tx::{ContractId, Receipt};
use futures::future::join_all;
use fuels::{
    accounts::provider::Provider,
    types::{
        transaction_response::TransactionResponse,
    },
};
use fuels::client::{PageDirection, PaginationRequest};
use fuels::prelude::{Transaction, TransactionType};
use fuels::types::BlockHeight;
use futures::{stream, StreamExt};
use tokio::task::JoinHandle;
use crate::config::CONFIG;
use crate::ports::blockchain::tx_sync::SwapEvent;

#[derive(Debug, Clone)]
pub struct LogEvent {
    pub transaction_hash: String,
    pub block_number: u32,
    pub contract_id: String,
    pub data: Vec<u8>,
}


#[repr(u64)]
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MiraEvent {
    Swap = 7938487056892321597,
    CreatePool = 12224862438738316526,
    TotalSupply = 17462098202904023478,
}

impl MiraEvent {
    pub fn from_u64(value: u64) -> Option<Self> {
        match value {
            x if x == MiraEvent::Swap as u64 => Some(MiraEvent::Swap),
            x if x == MiraEvent::CreatePool as u64 => Some(MiraEvent::CreatePool),
            x if x == MiraEvent::TotalSupply as u64 => Some(MiraEvent::TotalSupply),
            _ => None,
        }
    }

    pub fn as_u64(self) -> u64 {
        self as u64
    }
}

pub struct FuelRpcService {
    providers: Vec<Provider>,
    cache: Arc<Mutex<HashMap<String, LogEvent>>>
}

impl FuelRpcService {
    pub async fn new() -> Result<Self, fuels::types::errors::Error> {
        let provider1 = Provider::connect(CONFIG.default.rpc_url.as_str()).await?;
        let provider2 = Provider::connect(CONFIG.default.rpc_url.as_str()).await?;

        Ok(FuelRpcService {
            providers: vec![provider1, provider2],
            cache: Arc::new(Mutex::new(HashMap::new()))
        })
    }

    pub async fn get_logs_by_block_number(&self, provider: &Provider, block_number: u32) -> Result<Vec<SwapEvent>, fuels::types::errors::Error> {

        //log::info!("Block: {}", block_number);

        let block = provider.block_by_height(BlockHeight::from(block_number)).await?.unwrap();
        log::info!("block: {} : {}", block_number, block.transactions.len());

        let mut logs = Vec::new();
        //let mut rows: Vec<u32> = Vec::new();

        for tx in block.transactions {
            if let Some(tx_response) = provider.get_transaction_by_id(&tx).await? {
                let tr = tx_response.transaction.clone();
                let receipts = tx_response.status.clone().take_receipts();
                //log::info!("Block: {} - Tx: {:?}", block_number, tr.clone());
                //rows.push(block_number);
                match tr {
                    TransactionType::Script(script_tx) => {
                        //log::info!("Type: STX");
                        let mira_contract_id = ContractId::from_str(CONFIG.default.cdi_mira_amm.as_str())?;
                        for input in script_tx.inputs() {
                            let cid = input.contract_id();
                            if cid.is_some() {
                                if mira_contract_id == cid.unwrap().clone() {
                                    for receipt in receipts.clone() {
                                        match receipt.clone() {
                                            Receipt::LogData {
                                                id,
                                                ra,
                                                rb,
                                                ptr,
                                                len,
                                                digest,
                                                pc,
                                                is,
                                                data,
                                            } => {
                                                let log_id = receipt.rb().unwrap() as u64;

                                                match MiraEvent::from_u64(log_id) {
                                                    Some(MiraEvent::Swap) => {
                                                        let event = SwapEvent::try_from(receipt.data().unwrap()).unwrap();
                                                        logs.push(event);
                                                    },
                                                    _ => {}
                                                }
                                            },
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    },
                    TransactionType::Mint(mint_tx) => {
                        //log::info!("Type: Mint");
                    },
                    TransactionType::Create(create_tx) => {
                      //log::info!("Type: Create");
                    },
                    _ => {
                        //log::info!("Type: Unknown");
                    }
                }
            }
            else{
                log::info!("Tx not found");
            }
        }
        if(logs.len() > 0) {
            log::info!("Swaps in logs: {}", logs.len());
        }
        Ok(logs)
    }
    pub async fn get_logs_from_block_range(&self, block_number_start: u32, block_number_end: u32){

        let start_time = Instant::now();
/*        for block_number in block_number_start..=block_number_end {
            self.get_logs_by_block_number(block_number).await;
        }*/

        let concurrent_requests = 3;
        //let provider = self.provider.clone();

        let results = stream::iter(block_number_start..=block_number_end)
            .map(|block_number| {
                //let provider = provider.clone();
                let provider = self.providers[block_number_start as usize % self.providers.len()].clone();
                async move {
                    match self.get_logs_by_block_number(&provider, block_number).await {
                        Ok(logs) => Ok(logs),
                        Err(e) => {
                            log::error!("Error processing block {}: {:?}", block_number, e);
                            Err(e)
                        }
                    }
                }
            })
            .buffer_unordered(concurrent_requests)
            .collect::<Vec<_>>()
            .await;

        let mut all_logs = Vec::new();
        for result in results {
            if let Ok(logs) = result {
                all_logs.extend(logs);
            }
        }

        let duration = start_time.elapsed();
        log::info!("Cache initialization took: {:?} cache size: {}", duration, all_logs.len());

    }
}