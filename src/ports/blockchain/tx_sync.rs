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
use uuid::Uuid;
use crate::config::CONFIG;
use crate::domain::entity::TokenEntity;
use crate::domain::service::persistence::{SyncStatusService, TokenPairsService, TokenService};
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
                                        //log::info!("MINT TX");
                                    },
                                    TransactionType::Script(script_tx) => {
                                        log::info!("SCRIPT TX");
                                        let mira_contract_id = ContractId::from_str(CONFIG.default.cdi_mira_amm.as_str())?;
                                        for input in script_tx.inputs() {
                                            let cid = input.contract_id();
                                            if cid.is_some() {
                                                if mira_contract_id == cid.unwrap().clone() {
                                                    for receipt in receipts.clone(){
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
                                                                        log::info!("SwapEvent");
                                                                        log::info!("BlockID: {}", block_height );
                                                                        log::info!("TX: {}",tx);
                                                                        log::info!("Input: {}",input.utxo_id().unwrap_or(&Default::default()));
                                                                        //log::info!("{:?}",receipt);
                                                                        let event = SwapEvent::try_from(receipt.data().unwrap()).unwrap();
                                                                        //log::info!("{:?}",event);
                                                                        if let Some(asset_0_id) = get_token_details_by_asset_id(&provider, &event.pool_id.0).await? {
                                                                            if let Some(asset_1_id) = get_token_details_by_asset_id(&provider, &event.pool_id.1).await?{
                                                                                log::info!("A0: {:?}",asset_0_id);
                                                                                log::info!("A0 amount: IN:{}, OUT:{}", &event.asset_0_in, &event.asset_0_out);
                                                                                log::info!("A1: {:?}",asset_1_id);
                                                                                log::info!("A1 amount: IN:{}, OUT:{}", &event.asset_1_in, &event.asset_1_out);

                                                                                //1. Find pair or if doesn't exist
                                                                                let token_pair = TokenPairsService::find_or_create_pair(&asset_0_id,&asset_1_id).await;
                                                                                //2. Create log


                                                                            }
                                                                        }else{
                                                                            continue;
                                                                        }
                                                                    }
                                                                    Some(MiraEvent::CreatePool) => {
                                                                        log::info!("CreatePoolEvent");
                                                                    }
                                                                    Some(MiraEvent::TotalSupply) => {
                                                                        log::info!("TotalSupplyEvent");
                                                                    }
                                                                    None => {
                                                                        log::info!("OtherType log_id: {}", log_id);
                                                                    }
                                                                }
                                                            }
                                                            _ => {
                                                                //log::info!("Other type: {:?}",receipt);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    TransactionType::Create(create_tx) => {
                                        //log::info!("CREATE TX");
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

async fn get_token_details_by_asset_id(provider: &Provider,asset_id: &AssetId) -> Result<Option<TokenEntity>>{

    let token = TokenService::find_by_address(&asset_id.to_string()).await.unwrap();

    if token.is_some(){
        log::info!("Token found in DB");
        Ok(token)
    }
    else{
        log::info!("Token not found");

        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        let contract_id = ContractId::from_str(FUEL_TOKEN_GATEWAY_CID).unwrap_or(ContractId::zeroed());
        let fuel_token_gateway = crate::ports::tx_monitor_poc::FuelTokenGateway::new(contract_id, wallet);

        //TODO: There has to be more efficient way to take all this data at once


        let benchContract = Bech32ContractId
        ::from(ContractId::from_str("0x0ceafc5ef55c66912e855917782a3804dc489fb9e27edfd3621ea47d2a281156")
            .unwrap_or(ContractId::zeroed()));

        let response = fuel_token_gateway.methods().name(asset_id.clone()).with_contract_ids(&[benchContract.clone(),
        ]).simulate(Execution::StateReadOnly).await;

        match response{
            Ok(call_response) => {
                match call_response.value {
                    Some(token_name) => {
                        let token_symbol = fuel_token_gateway.methods().symbol(asset_id.clone()).with_contract_ids(&[benchContract.clone(),
                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                        let token_decimals = fuel_token_gateway.methods().decimals(asset_id.clone()).with_contract_ids(&[benchContract.clone(),
                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                        let token_entity = TokenEntity{
                            id: Uuid::new_v4(),
                            address: asset_id.to_string(),
                            symbol: token_symbol,
                            name: token_name,
                            decimals: token_decimals as i32,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        };
                        Ok(Some(TokenService::create(token_entity).await.unwrap()))
                    },
                    None => {
                        log::info!("No asset found - int");
                        Ok(None)
                    },
                }
            }
            Err(e) => {
                log::info!("No asset found - ext");
                Ok(None)
            }
        }
    }

}
