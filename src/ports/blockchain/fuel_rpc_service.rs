use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use fuels::{
    accounts::provider::Provider,
};
use fuels::prelude::{Error, Transaction, TransactionType};
use fuels::tx::Receipt;
use fuels::types::{BlockHeight, ContractId};
use futures::{stream, StreamExt};
use crate::config::CONFIG;
use crate::ports::blockchain::fuel_model::Swap;
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
    //TODO - we have to clean this cache
    cache: Arc<Mutex<HashMap<String, Vec<Swap>>>>
}

impl FuelRpcService {
    pub async fn new() -> Result<Self, fuels::types::errors::Error> {

        let provider1= Provider::connect(CONFIG.default.rpc_url_one.as_str()).await?;
        let provider2= Provider::connect(CONFIG.default.rpc_url_two.as_str()).await?;
        let provider3= Provider::connect(CONFIG.default.rpc_url_three.as_str()).await?;

        Ok(FuelRpcService {
            providers: vec![provider1, provider2, provider3],
            cache: Arc::new(Mutex::new(HashMap::new()))
        })
    }

    pub async fn initialize_cache(&self, from_block: u32) -> Result<(), fuels::types::errors::Error> {
        let latest_block_number = self.providers[0].latest_block_height().await?;

        if from_block > latest_block_number {
            return Err(fuels::types::errors::Error::Provider(
                "Start block is higher than the latest block".into()
            ));
        }

        log::info!(
            "Initializing cache from block {} to {}",
            from_block,
            latest_block_number
        );

        self.get_logs_from_block_range(from_block, latest_block_number).await;
        Ok(())
    }

    pub async fn get_logs_by_block_number(&self, provider: &Provider, block_number: u32) -> Result<Vec<Swap>, fuels::types::errors::Error> {

        log::info!("Block: {}", block_number);

        let last_block = provider.block_by_height(BlockHeight::from(block_number)).await?;

        if last_block.is_none() {
            return Ok(Vec::new());
        }
        let block = last_block.unwrap();

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
                                                        let event = Swap{
                                                            tx_id:tx.clone().to_string(),
                                                            swap_event: SwapEvent::try_from(receipt.data().unwrap()).unwrap()
                                                        };
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
        if logs.len() > 0 {
            log::info!("Swaps in logs: {}", logs.len());
        }
        Ok(logs)
    }
    //TODO - should be removed?
    async fn get_logs_from_block_range(&self, block_number_start: u32, block_number_end: u32){

        let start_time = Instant::now();

        let concurrent_requests = 3;

        let results = stream::iter(block_number_start..=block_number_end)
            .map(|block_number| {
                let provider = self.providers[block_number_start as usize % self.providers.len()].clone();
                async move {
                    match self.get_logs_by_block_number(&provider, block_number).await {
                        Ok(logs) => {
                            self.cache.lock().unwrap().insert(block_number.to_string(), logs.clone());
                            Ok(logs)
                        },
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
        log::info!("Cache update took: {:?} cache size: {}", duration, self.cache.lock().unwrap().len() );

    }

    pub async fn get_logs(&self, requested_block: u32) -> Result<Vec<Swap>, fuels::types::errors::Error> {

        let logs = self.cache.lock().unwrap().get(&requested_block.to_string()).map(|v| v.clone()).unwrap_or(Vec::new());

        if logs.len() > 0 {
            log::info!("Swaps in cache: {}", logs.len());
            return Ok(logs);
        }
        else{
            // First check if block exists in cache
            let latest_cached_block = self.get_latest_cached_block();

            if let Some(cached_block) = latest_cached_block {
                if requested_block <= cached_block {
                    // We already have this block in cache
                    let provider = &self.providers[0];
                    return self.get_logs_by_block_number(provider, requested_block).await;
                }
            }

            // If we get here, we need to update the cache
            // Now we need to check the latest block from the blockchain
            let latest_block_number = self.providers[0].latest_block_height().await?;

            if requested_block > latest_block_number {
                log::info!("latest_block_number: {}", latest_block_number);
                log::info!("requested_block: {}", requested_block);
                return Err(fuels::types::errors::Error::Provider(
                    "Requested block is higher than the latest block".into()
                ));
            }

            // Update cache from the last cached block (or requested block if cache is empty)
            let start_block = latest_cached_block.map(|b| b + 1).unwrap_or(requested_block);

            log::info!(
            "Updating cache from block {} to {}",
            start_block,
            latest_block_number
        );

            self.get_logs_from_block_range(start_block, latest_block_number).await;

            // Return the logs for the requested block
            let provider = &self.providers[0];
            self.get_logs_by_block_number(provider, requested_block).await
        }
    }

    fn get_latest_cached_block(&self) -> Option<u32> {
        let cache = self.cache.lock().unwrap();
        cache.keys()
            .map(|k| k.parse::<u32>().unwrap_or(0))
            .max()
    }
}