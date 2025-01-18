use std::net::ToSocketAddrs;
use std::str::FromStr;
use fuels::prelude::*;
use std::time::Duration;
use chrono::{DateTime, Utc};
use fuels::tx::TxParameters;
use fuels::types::BlockHeight;
use fuel_tx::Input;
use fuels::core::codec::{ABIDecoder, DecoderConfig};
use fuels::types::coin_type_id::CoinTypeId::UtxoId;
use fuels::types::output::Output;
use fuels::types::param_types::ParamType;
use num_traits::AsPrimitive;
use serde::Deserialize;
use crate::config::CONFIG;
use crate::domain::service::persistence::SyncStatusService;
use crate::ports::blockchain::blockchain_data_service::BlockchainDataService;
use crate::ports::db::database_manager::DB_MANAGER;
use crate::ports::tx_monitor_poc::MiraEvent;

pub struct TxSync;

abigen!(
    Contract(
       name = "FuelTokenGateway",
        abi = "resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json",
    ),
    Contract(
       name = "MiraV1Core",
        abi = "resources/abi/mira_amm_contract/out/debug/mira_amm_contract-abi.json"
    ),
);
static FUEL_TOKEN_GATEWAY_CID: &str = "0x4ea6ccef1215d9479f1024dff70fc055ca538215d2c8c348beddffd54583d0e8";
static MIRA_AMM_CID: &str = "0x4ea6ccef1215d9479f1024dff70fc055ca538215d2c8c348beddffd54583d0e8";
static MIRA_AMM_2_CID: &str = "0x2e40f2b244b98ed6b8204b3de0156c6961f98525c8162f80162fcf53eebd90e7";


impl TxSync{
    pub async fn synchronize_transactions() -> Result<()> {
        let provider = Provider::connect("https://mainnet.fuel.network").await?;
        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        let mut start_block:u32 = get_start_block_number().await;
        log::info!(" TXS: Starting from block: {}",start_block);

        let start_block_time = get_block_time_by_block_height(&provider, start_block).await;

        log::info!("TXS - Start block time: {:?}",start_block_time);

        loop {
            let current_block = provider.latest_block_height().await?;

            if current_block > start_block {
                for block_height in start_block..=current_block {

                    if is_block_in_calc_window(&provider, block_height as u64).await {
                        let block = provider.block_by_height(BlockHeight::from(block_height)).await?;

                        if let Some(block) = block {
                            for tx in block.transactions {
                                let txr = provider.get_transaction_by_id(&tx).await?.unwrap();
                                let transaction = txr.transaction.clone();
                                let receipts = txr.status.clone().take_receipts();
                                match transaction {
                                    TransactionType::Mint(mint_tx) => {
                                        log::info!("MINT TX");
                                    },
                                    TransactionType::Script(script_tx) => {
                                        log::info!("SCRIPT TX");
                                    },
                                    TransactionType::Create(create_tx) => {
                                        log::info!("CREATE TX");
                                    },

                                    _ => {
                                        log::info!("Other TX type");
                                    }
                                }
                            }
                        }
                    }
                    else{
                        log::info!("Block {} out of calc window - skipped",block_height);
                    }

                }
            }
            //break;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }

}
async fn get_block_time_by_block_height(provider: &Provider, block_height: u32) -> fuels::prelude::Result<DateTime<Utc>> {
    let block = provider.block_by_height(BlockHeight::new(block_height.clone())).await?;
    Ok(block.unwrap().header.time.unwrap())
}

async fn get_start_block_number() ->u32 {
    let mut block_number = 0;
    match SyncStatusService::get_status_entity().await {
        Ok(Some(sync_status_entity)) => {
            block_number = sync_status_entity.block_number as u32
        },
        Ok(None) => { block_number = 0;},
        Err(_) => { block_number = 0; },//TODO - Exception management
    }
    if block_number == 0{
        block_number = CONFIG.default.tx_log_start_block_number as u32;
    }
    block_number
}

async fn is_block_in_calc_window(provider: &Provider, block_number: u64) -> bool{
    // Fetch the block time
    let block_time_result = BlockchainDataService::get_block_time(provider, &block_number).await;

    // Check if block_time was successfully fetched
    let block_time = match block_time_result {
        Ok(time) => time,
        Err(_) => {
            // If fetching block_time fails, assume it's out of range
            return false;
        }
    };

    // Get the calculation window in hours from the config
    let window_range_hours = CONFIG.default.calculation_window as i64;

    // Calculate the cutoff time
    let cutoff_time = Utc::now() - Duration::from_hours(window_range_hours as u64);

    // Check if the block_time is within the calculation window
    block_time >= cutoff_time
}




