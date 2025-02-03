use std::collections::{HashMap, HashSet};
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
use log::error;
use num_traits::AsPrimitive;
use sea_orm::DbErr;
use sea_orm::prelude::Decimal;
use serde::Deserialize;
use uuid::Uuid;
use crate::config::CONFIG;
use crate::domain::entity::{TokenEntity, TokenPairsEntity, UnknownTokenEntity};
use crate::domain::entity::mira_pools_entity::MiraPoolsEntity;
use crate::domain::entity::pair_swaps_entity::PairSwapsEntity;
use crate::domain::service::persistence::{PairSwapsService, SyncStatusService, TokenPairsService, TokenService, UnknownTokenService};
use crate::domain::service::persistence::mira_pools_service::MiraPoolsService;
use crate::ports::blockchain::blockchain_data_service::BlockchainDataService;
use crate::ports::db::database_manager::DB_MANAGER;
use crate::ports::db::model::prelude::PairSwaps;
use crate::ports::db::model::unknown_token;
use crate::ports::sentio::{Pool, SubgraphQueryService};

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

impl TxSync{
    pub async fn synchronize_transactions(runner_id: u8) -> Result<()> {
        let provider = Provider::connect(CONFIG.default.rpc_url.as_str()).await?;
        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        //let mut start_block:u32 = provider.latest_block_height().await?; // get_start_block_number().await;
        let mut start_block:u32 = get_start_block_number().await;
        log::info!(" TXS-{}: Starting from block: {}",runner_id,start_block);
        let start_block_time = get_block_time_by_block_height(&provider, start_block).await;

        log::info!("TXS-{}: - Start block time: {:?}",runner_id,start_block_time);

        loop {
            let current_block = provider.latest_block_height().await?;

            let subgraph_service = SubgraphQueryService::new();
            let _ = subgraph_service.initialize_cache(start_block,current_block).await;

            log::info!("TXS-{}: - Current block: {}",runner_id,current_block);

            if current_block > start_block {

                let mut updated_pairs: HashMap<Uuid, TokenPairsEntity> = HashMap::new();

                for block_height in start_block..=current_block {
                    log::info!("TXS-{}: - Block {} - Start",runner_id,block_height);

                    if is_block_in_calc_window(&provider, block_height as u64).await {
                        let block = provider.block_by_height(BlockHeight::from(block_height)).await?;

                        if let Some(block) = block {
                            if PairSwapsService::exists_by_block_number(block_height as i32).await{
                                log::info!("TXS-{}: - Block {} - PairSwaps already exists - skipped",runner_id,block_height);
                                continue;
                            }

                            let mut pair_swaps_vec: Vec<PairSwapsEntity> = Vec::new();
                            let block_time = BlockchainDataService::get_block_time(&provider, &(block_height as u64)).await.unwrap();


                            //let swaps = subgraph_service.get_logs_by_block_number(block_height).await.unwrap_or_else(|_| Vec::new());;

                            let swaps = subgraph_service.get_logs_by_block_number_from_cache(block_height);

                            if !swaps.is_empty(){
                                for swap in swaps {
                                    //swap.
                                    //log::info!("Swap: {:?}",swap);

                                    let pool = Pool::from_pool_id(&swap.pool_id).unwrap();
                                    //log::info!("Pool: {:?}",pool);
                                    let token_base
                                        = get_mira_token_details_by_asset_id(&provider,&AssetId::from_str(pool.token0_address.as_str()).unwrap()).await.unwrap_or(None);
                                    if let Some(ref token_base) = token_base{
                                        let token_quote
                                            = get_mira_token_details_by_asset_id(&provider,&AssetId::from_str(pool.token1_address.as_str()).unwrap()).await.unwrap_or(None);

                                        if let Some(ref token_quote) = token_quote {
                                            let token_pair = find_or_create_pair(
                                                token_base,token_quote).await.unwrap();
                                            updated_pairs.insert(token_pair.id,token_pair.clone());

                                            let pair_swap = PairSwapsEntity{
                                                id: Uuid::new_v4(),
                                                block_number: block_height.to_string(),
                                                block_time: Some(block_time),
                                                tx_id: swap.transaction_hash,
                                                utxo_id: "".to_string(),
                                                pair_id: token_pair.id,
                                                base_amount: Decimal::from(swap.token_0in.parse::<u64>().unwrap()),
                                                quote_amount: Decimal::from(swap.token_1out.parse::<u64>().unwrap()),
                                                created_at: Utc::now(),
                                                updated_at: Utc::now(),
                                            };
                                            pair_swaps_vec.push(pair_swap);

                                        }
                                    }
                                }
                                log::info!("TXS-{}: - Block {} - PairSwaps: {}",runner_id,block_height,pair_swaps_vec.len());
                                let _ = PairSwapsService::create_many_with_sync(pair_swaps_vec, block_height as i32,block_time).await;
                            }else {
                                log::info!("TXS-{}: - Block {} - No swaps found - skipped",runner_id,block_height);
                                continue;
                            }

                            /*

                            for tx in block.transactions {
                                let txr = provider.get_transaction_by_id(&tx).await?.unwrap();
                                let transaction = txr.transaction.clone();
                                let receipts = txr.status.clone().take_receipts();
                                match transaction {
                                    TransactionType::Mint(mint_tx) => {
                                        //log::info!("MINT TX");
                                    },
                                    TransactionType::Script(script_tx) => {
                                        log::info!("TXS-{}: SCRIPT TX",runner_id);
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
                                                                        log::info!("TXS-{}: SwapEvent",runner_id);
                                                                        log::info!("TXS-{}: BlockID: {}",runner_id, block_height );
                                                                        log::info!("TXS-{}: TX: {}",runner_id,tx);
                                                                        log::info!("TXS-{}: Input: {}",runner_id,input.utxo_id().unwrap_or(&Default::default()));
                                                                        //log::info!("{:?}",receipt);
                                                                        let event = SwapEvent::try_from(receipt.data().unwrap()).unwrap();
                                                                        //log::info!("{:?}",event);
                                                                        if let Some(asset_0_id) = get_token_details_by_asset_id(&provider, &event.pool_id.0).await? {
                                                                            if let Some(asset_1_id) = get_token_details_by_asset_id(&provider, &event.pool_id.1).await?{
                                                                                log::info!("TXS-{}: A0: {:?}",runner_id,asset_0_id);
                                                                                log::info!("TXS-{}: A0 amount: IN:{}, OUT:{}",runner_id, &event.asset_0_in, &event.asset_0_out);
                                                                                log::info!("TXS-{}: A1: {:?}",runner_id,asset_1_id);
                                                                                log::info!("TXS-{}: A1 amount: IN:{}, OUT:{}",runner_id, &event.asset_1_in, &event.asset_1_out);

                                                                                //1. Find pair or if doesn't exist
                                                                                let token_pair = find_or_create_pair(&asset_0_id, &asset_1_id).await;
                                                                                //2. Create log
                                                                                let pair_swap = PairSwapsEntity{
                                                                                    id: Uuid::new_v4(),
                                                                                    block_number: block_height.to_string(),
                                                                                    block_time: Some(block_time),
                                                                                    tx_id: tx.to_string(),
                                                                                    utxo_id: input.utxo_id().unwrap_or(&Default::default()).to_string(),
                                                                                    pair_id: token_pair.unwrap().id,
                                                                                    base_amount: Decimal::from(event.asset_0_in.clone()),
                                                                                    quote_amount: Decimal::from(event.asset_1_out.clone()),
                                                                                    created_at: Utc::now(),
                                                                                    updated_at: Utc::now(),
                                                                                };
                                                                                pair_swaps_vec.push(pair_swap);

                                                                            }
                                                                        }else{
                                                                            continue;
                                                                        }
                                                                    }
                                                                    Some(MiraEvent::CreatePool) => {
                                                                        log::info!("TXS-{}: CreatePoolEvent",runner_id);
                                                                    }
                                                                    Some(MiraEvent::TotalSupply) => {
                                                                        log::info!("TXS-{}: TotalSupplyEvent",runner_id);
                                                                    }
                                                                    None => {
                                                                        log::info!("TXS-{}: OtherType log_id: {}",runner_id, log_id);
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
                                        log::info!("TXS-{}: Other TX type",runner_id);
                                    }
                                }
                            }
                            log::info!("TXS-{}: - Block {} - PairSwaps: {}",runner_id,block_height,pair_swaps_vec.len());
                            let _ = PairSwapsService::create_many_with_sync(pair_swaps_vec, block_height as i32,block_time).await;

                             */
                        }
                    }
                    else{
                        log::info!("TXS-{}: Block {} out of calc window - skipped",runner_id,block_height);
                    }

                }

                if updated_pairs.len() > 0{
                    for (id,pair) in &updated_pairs{
                        find_or_create_mira_pool(pair.id).await;
                    }
                }
            }
            //break;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }


}
async fn find_or_create_pair(base_token: &TokenEntity, quote_token: &TokenEntity) -> Option<TokenPairsEntity> {
    match TokenPairsService::find_or_create_pair(&base_token, &quote_token).await{
        Ok(token_pair) =>{
            //let _ = find_or_create_mira_pool(token_pair.id).await;
            Some(token_pair)
        }
        Err(err)=>{
            log::error!("Error while finding or creating token pair: {}", err);
            None
        }
    }

}

async fn find_or_create_mira_pool(pair_id: Uuid) -> Option<MiraPoolsEntity>{

    match MiraPoolsService::find_or_create(pair_id).await{
        Ok(mira_pool)=>{
            //refresh
            Some(refresh_mira_pool(mira_pool).await)
        }
        Err(err)=>{
            None
        }
    }


}

async fn refresh_mira_pool(mira_pool: MiraPoolsEntity) -> MiraPoolsEntity{
    get_mira_pool_metadata(mira_pool).await
}

async fn get_block_time_by_block_height(provider: &Provider, block_height: u32) -> fuels::prelude::Result<DateTime<Utc>> {
    let block = provider.block_by_height(BlockHeight::new(block_height.clone())).await?;
    Ok(block.unwrap().header.time.unwrap())
}

async fn get_start_block_number() ->u32 {
    let mut block_number = 0;
    match SyncStatusService::get_status_entity().await {
        Ok(Some(sync_status_entity)) => {
            block_number = sync_status_entity.block_number as u32 +1
        },
        Ok(None) => { block_number = 0;},
        Err(_) => { block_number = 0; },//TODO - Exception management
    }
    if block_number <= 1{
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

    log::info!("Fetching token details by asset_id: {}",asset_id.to_string());
    let token = TokenService::find_by_address(&asset_id.to_string()).await.unwrap();

    if token.is_some(){
        log::info!("Token found in DB");
        Ok(token)
    }
    else{
        log::info!("Token not found - fetching from gateway ....");

        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        let contract_id = ContractId::from_str(CONFIG.default.cdi_fuel_token_gateway.as_str()).unwrap_or(ContractId::zeroed());
        let fuel_token_gateway = FuelTokenGateway::new(contract_id, wallet);

        //TODO: There has to be more efficient way to take all this data at once


        let benchContract = Bech32ContractId
        ::from(ContractId::from_str(CONFIG.default.cdi_fuel_token_gateway_dependency.as_str())
            .unwrap_or(ContractId::zeroed()));

        let response = fuel_token_gateway.methods().name(asset_id.clone()).with_contract_ids(&[benchContract.clone(),
        ]).simulate(Execution::StateReadOnly).await;
        //log::info!("TOKEN FROM GATEWAY {:?}",response);

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
                        log::info!("All data ready to create new Token entity: {:?}",token_entity);
                        Ok(Some(TokenService::create(token_entity).await.unwrap()))
                    },
                    None => {
                        log::info!("No asset found - int - switch to Mira");
                        if let Some(token_entity) = get_mira_token_details_by_asset_id(provider,asset_id).await?{
                         Ok(Some(token_entity))
                        }else{
                            Ok(None)
                        }

                    },
                }
            }
            Err(e) => {
                log::info!("No asset found - ext - switch to Mira");

                if let Some(token_entity) = get_mira_token_details_by_asset_id(provider,asset_id).await?{
                    Ok(Some(token_entity))
                }else{
                    let unknown_token = UnknownTokenEntity{
                        id: Uuid::new_v4(),
                        address: asset_id.to_string(),
                    };
                    let _ = UnknownTokenService::create_if_not_exists(unknown_token).await;
                    Ok(None)
                }
            }
        }
    }

}

async fn get_mira_token_details_by_asset_id(provider: &Provider,asset_id: &AssetId) -> Result<Option<TokenEntity>>{

    log::info!("Fetching mira token details by asset_id: {}",asset_id.to_string());
    let token = TokenService::find_by_address(&asset_id.to_string()).await.unwrap();

    if token.is_some(){
        log::info!("Token found in DB");
        Ok(token)
    }
    else{
        log::info!("Token not found - fetching from gateway ....");

        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        let mira_cid = ContractId::from_str(CONFIG.default.cdi_mira_token_gateway.as_str())
            .unwrap_or(ContractId::zeroed());
        let mira_contract = MiraV1Core::new(mira_cid, wallet.clone());

        //TODO: There has to be more efficient way to take all this data at once

        let response = mira_contract.methods().name(asset_id.clone())
            //.with_contract_ids(&[bench_contract.clone(), ])
        .simulate(Execution::StateReadOnly).await;
        //log::info!("MIRA TOKEN FROM GATEWAY {:?}",response);

        match response{
            Ok(call_response) => {
                match call_response.value {
                    Some(token_name) => {
                        log::info!("MIRA TOKEN NAME: {:?}",token_name);
                        let token_symbol = mira_contract.methods().symbol(asset_id.clone()).simulate(Execution::StateReadOnly).await?.value.unwrap();
                        log::info!("MIRA TOKEN SYMBOL: {:?}",token_symbol);
                        let token_decimals = mira_contract.methods().decimals(asset_id.clone()).simulate(Execution::StateReadOnly).await?.value.unwrap();
                        log::info!("MIRA TOKEN DECIMALS: {:?}",token_decimals);

                        let token_entity = TokenEntity{
                            id: Uuid::new_v4(),
                            address: asset_id.to_string(),
                            symbol: token_symbol,
                            name: token_name,
                            decimals: token_decimals as i32,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                        };
                        log::info!("Mira - All data ready to create new Token entity: {:?}",token_entity);
                        Ok(Some(TokenService::create(token_entity).await.unwrap()))
                    },
                    None => {
                        log::info!("Mira - No asset found in TG");
                        log::info!("Checking Fuel Token Gateway ....");
                        let fuel_contract = ContractId::from_str(CONFIG.default.cdi_fuel_token_gateway.as_str())
                            .unwrap_or(ContractId::zeroed());
                        let fuel_gateway = MiraV1Core::new(fuel_contract, wallet);
                        let bench_contract = Bech32ContractId
                        ::from(ContractId::from_str(CONFIG.default.cdi_fuel_token_gateway_dependency.as_str())
                            .unwrap_or(ContractId::zeroed()));
                        let response = fuel_gateway.methods().name(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),])
                            .simulate(Execution::StateReadOnly).await;

                        match response{
                            Ok(call_response) => {
                                match call_response.value {
                                    Some(token_name) => {
                                        log::info!("I have token from fuel gateway");

                                        log::info!("FUEL TOKEN NAME: {:?}",token_name);
                                        let token_symbol = fuel_gateway.methods().symbol(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
                                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();
                                        log::info!("FUEL TOKEN SYMBOL: {:?}",token_symbol);
                                        let token_decimals = fuel_gateway.methods().decimals(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
                                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();
                                        log::info!("FUEL TOKEN DECIMALS: {:?}",token_decimals);

                                        let token_entity = TokenEntity {
                                            id: Uuid::new_v4(),
                                            address: asset_id.to_string(),
                                            symbol: token_symbol,
                                            name: token_name,
                                            decimals: token_decimals as i32,
                                            created_at: Utc::now(),
                                            updated_at: Utc::now(),
                                        };
                                        log::info!("Fuel - All data ready to create new Token entity: {:?}",token_entity);
                                        Ok(Some(TokenService::create(token_entity).await.unwrap()))
                                    },
                                    None => {
                                        log::info!("No token from fuel gateway");
                                        let unknown_token = UnknownTokenEntity {
                                            id: Uuid::new_v4(),
                                            address: asset_id.to_string(),
                                        };
                                        let _ = UnknownTokenService::create_if_not_exists(unknown_token).await;
                                        Ok(None)
                                    }
                                }
                            },
                            Err(e)=>{
                                log::info!("Fuel - No asset found - ext L1");
                                let unknown_token = UnknownTokenEntity{
                                    id: Uuid::new_v4(),
                                    address: asset_id.to_string(),
                                };
                                let _ = UnknownTokenService::create_if_not_exists(unknown_token).await;
                                Ok(None)
                            }
                        }
                        }
                    }
            }
            Err(e) => {
                log::info!("Mira - No asset found - ext L0");
                let unknown_token = UnknownTokenEntity{
                    id: Uuid::new_v4(),
                    address: asset_id.to_string(),
                };
                let _ = UnknownTokenService::create_if_not_exists(unknown_token).await;
                Ok(None)
            }
        }
    }

}


async fn get_mira_pool_metadata(mut pool: MiraPoolsEntity) -> MiraPoolsEntity {
    match TokenPairsService::find_by_id(pool.pair_id).await {
        Ok(Some(token_pair)) => {
            if let Ok(provider) = Provider::connect(CONFIG.default.rpc_url.as_str()).await {
                let mut wallet = WalletUnlocked::new_random(None);
                wallet.set_provider(provider.clone());

                let mira_cid = ContractId::from_str(CONFIG.default.cdi_mira_amm.as_str())
                    .unwrap_or(ContractId::zeroed());

                let mira_contract = MiraV1Core::new(mira_cid, wallet);

                match mira_contract.methods()
                    .pool_metadata(
                        (
                            AssetId::from_str(token_pair.base_address.as_str())
                                .unwrap_or_default(),
                            AssetId::from_str(token_pair.quote_address.as_str())
                                .unwrap_or_default(),
                            false,
                        )
                    )
                    .simulate(Execution::StateReadOnly)
                    .await
                {
                    Ok(response) => {
                        if let Some(pool_sample) = response.value {
                            pool.reserve_base = Decimal::new(pool_sample.reserve_0 as i64, 0);
                            pool.reserve_quote = Decimal::new(pool_sample.reserve_1 as i64, 0);
                            pool.updated_at = Utc::now();

                            log::info!("Updating pool metadata {:?}", pool);

                            match MiraPoolsService::update(pool.clone()).await {
                                Ok(result) => {
                                    log::info!("Update successful: {:?}", result);
                                }
                                Err(err) => {
                                    log::error!("Update error: {}", err);
                                }
                            }
                        } else {
                            log::error!("Failed to get pool metadata: No value returned");
                        }
                    }
                    Err(err) => {
                        log::error!("Contract simulation error: {:?}", err);
                    }
                }
            } else {
                log::error!("Failed to connect to the provider");
            }
        }
        Ok(None) => {
            log::error!("Token pair not found for ID: {}", pool.pair_id);
        }
        Err(err) => {
            log::error!("Error fetching token pair: {:?}", err);
        }
    }

    pool
}
