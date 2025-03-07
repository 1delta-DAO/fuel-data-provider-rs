use std::collections::HashMap;
use std::str::FromStr;
use std::time::{Duration, Instant};
use chrono::{DateTime, Utc};
use fuels::prelude::{abigen, Bech32ContractId, Error, Execution, Provider, WalletUnlocked};
use fuels::types::{AssetId, BlockHeight, ContractId};
use num_traits::ToPrimitive;
use sea_orm::DbErr;
use sea_orm::prelude::Decimal;
use uuid::Uuid;
use crate::config::CONFIG;
use crate::domain::entity::{PriceDataEntity, TokenEntity, TokenPairsEntity, UnknownTokenEntity, VolumeDataEntity};
use crate::domain::entity::mira_pools_entity::MiraPoolsEntity;
use crate::domain::entity::pair_swaps_entity::PairSwapsEntity;
use crate::domain::service::persistence::{PairSwapsService, PriceDataService, SyncStatusService, TokenPairsService, TokenService, UnknownTokenService, VolumeDataService};
use crate::domain::service::persistence::mira_pools_service::MiraPoolsService;
use crate::ports::blockchain::blockchain_data_service::BlockchainDataService;
use crate::ports::blockchain::fuel_model::Pool;
use crate::ports::blockchain::FuelRpcService;
use crate::ports::db::model::token::Column::Price;

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
    pub async fn synchronize_transactions(runner_id: u8) -> Result<(), Error> {
        let provider = Provider::connect(CONFIG.default.rpc_url_one.as_str()).await?;
        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        //let mut start_block:u32 = provider.latest_block_height().await?; // get_start_block_number().await;
        let start_block:u32 = get_start_block_number().await;
        log::info!("TXS-{}: Starting from block: {}",runner_id,start_block);
        let start_block_time = get_block_time_by_block_height(&provider, start_block).await;

        log::info!("TXS-{}: - Start block time: {:?}",runner_id,start_block_time);

        let fuel_rpc_service = FuelRpcService::new().await?;
        //let _ = fuel_rpc_service.initialize_cache(start_block).await?;
        //let subgraph_service = SubgraphQueryService::new();

        loop {
            let current_block = provider.latest_block_height().await?;

            log::info!("TXS-{}: - Current block: {}",runner_id,current_block);

/*            for block_height in start_block..=current_block {
                if is_block_in_calc_window(&provider, block_height as u64).await {
                    let _ = fuel_rpc_service.initialize_cache(block_height).await?;
                    start_block = block_height;
                    break;
                }
            }*/


            //return Ok(());

            //let _ = subgraph_service.initialize_cache(start_block,current_block).await;



            if current_block > start_block {

                let mut updated_pairs: HashMap<Uuid, TokenPairsEntity> = HashMap::new();

                for block_height in start_block..=current_block {
                    let start = Instant::now();
                    //log::info!("TXS-{}: - Block {} - Start",runner_id,block_height, start.elapsed());

                    if is_block_in_calc_window(&provider, block_height as u64).await {
                        //let block = provider.block_by_height(BlockHeight::from(block_height)).await?;
                        //log::info!("TXS-{}: - Block {} - Start - block found {:?}",runner_id,block_height, start.elapsed());
                        let mut pair_swaps_vec: Vec<PairSwapsEntity> = Vec::new();

                        if PairSwapsService::exists_by_block_number(block_height as i32).await{
                            log::info!("TXS-{}: - Block {} - PairSwaps already exists - skipped - ut:{:?}",runner_id,block_height, start.elapsed());
                            continue;
                        }

                        let swaps = fuel_rpc_service.get_logs(block_height).await?;
                        //log::info!("TXS-{}: - Block {} - Swaps: {} ut:{:?}",runner_id,block_height,swaps.len(), start.elapsed());

                            if !swaps.is_empty(){

                                log::info!("Block {} - Swaps: {}",block_height,swaps.len());

                                let block_time = BlockchainDataService::get_block_time(&provider, &(block_height as u64)).await.unwrap();

                                for swap in swaps {

                                    let pool = Pool::from_swap(&swap.swap_event).unwrap();
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
                                                tx_id: swap.tx_id,
                                                utxo_id: "".to_string(),
                                                pair_id: token_pair.id,
                                                base_amount: swap.swap_event.asset_0_in,
                                                quote_amount: swap.swap_event.asset_1_out,
                                                created_at: Utc::now(),
                                                updated_at: Utc::now(),
                                            };

                                            if swap.swap_event.asset_0_in !=0
                                            {
                                                add_volume(token_base,token_quote,&pair_swap).await.unwrap();
                                                add_price(token_base,token_quote,&pair_swap).await.unwrap();
                                            }
                                            else{
                                                log::info!("Amount: {},{} : {},{}",swap.swap_event.asset_0_in,swap.swap_event.asset_0_in,swap.swap_event.asset_1_out,swap.swap_event.asset_1_out);
                                            }

                                            pair_swaps_vec.push(pair_swap);

                                        }
                                    }
                                }
                                log::info!("TXS-{}: - Block {} - PairSwaps: {}",runner_id,block_height,pair_swaps_vec.len());
                                let _ = PairSwapsService::create_many_with_sync(pair_swaps_vec, block_height as i32,block_time).await;
                            }else {
                                //log::info!("TXS-{}: - Block {} - No swaps found - skipped - ut:{:?}",runner_id,block_height, start.elapsed());
                                continue;
                            }
                        //}
                    }
                    else{
                        log::info!("TXS-{}: Block {} out of calc window - skipped - ut:{:?}",runner_id,block_height, start.elapsed());
                    }

                }

                if updated_pairs.len() > 0{
                    for (_id,pair) in &updated_pairs{
                        find_or_create_mira_pool(pair.id).await;
                    }
                }
            }
            //break;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }


}

pub async fn add_volume(
    token_base: &TokenEntity,
    token_quote: &TokenEntity,
    pair_swap: &PairSwapsEntity,
) -> Result<(), DbErr> {
    let timestamp = match pair_swap.block_time {
        Some(time) => time,
        None => {
            log::error!("Missing block_time for PairSwapsEntity: {:?}", pair_swap);
            return Err(DbErr::Custom("Missing block_time".to_string()));
        }
    };

    let volume_base = VolumeDataEntity {
        timestamp,
        token_id: token_base.id,
        volume: pair_swap.base_amount.to_u64().unwrap_or_else(|| {
            log::error!("Failed to convert base_amount for token: {:?}", token_base);
            0
        }),
    };

    if let Err(err) = VolumeDataService::create_or_update(volume_base).await {
        log::error!(
            "Failed to update volume for base token {}: {:?}",
            token_base.id, err
        );
        return Err(err);
    }

    let volume_quote = VolumeDataEntity {
        timestamp,
        token_id: token_quote.id,
        volume: pair_swap.quote_amount.to_u64().unwrap_or_else(|| {
            log::error!("Failed to convert quote_amount for token: {:?}", token_quote);
            0
        }),
    };

    if let Err(err) = VolumeDataService::create_or_update(volume_quote).await {
        log::error!(
            "Failed to update volume for quote token {}: {:?}",
            token_quote.id, err
        );
        return Err(err);
    }

    Ok(())
}

pub async fn add_price(
    token_base: &TokenEntity,
    token_quote: &TokenEntity,
    pair_swap: &PairSwapsEntity,
) -> Result<(), DbErr> {
    let timestamp = match pair_swap.block_time {
        Some(time) => time,
        None => {
            log::error!("Missing block_time for PairSwapsEntity: {:?}", pair_swap);
            return Err(DbErr::Custom("Missing block_time".to_string()));
        }
    };

    match (token_base.quoting, token_quote.quoting) {
        (true, false) => {
            // token_base is quoting, calculate price of token_quote
            let price = pair_swap.base_amount as f64/ pair_swap.quote_amount as f64;
            update_token_price(token_base, price, timestamp).await?;
        }
        (false, true) => {
            // token_quote is quoting, calculate price of token_base
            let price = pair_swap.quote_amount as f64 / pair_swap.base_amount as f64;
            update_token_price(token_quote, price, timestamp).await?;
        }
        (true, true) => {
            // Both tokens are quoting, assign reciprocal prices
            let base_price = pair_swap.base_amount as f64 / pair_swap.quote_amount as f64;
            let quote_price = pair_swap.quote_amount as f64 / pair_swap.base_amount as f64;
            update_token_price(token_base, base_price, timestamp).await?;
            update_token_price(token_quote, quote_price, timestamp).await?;
        }
        (false, false) => {
            log::warn!(
                "Skipping price update: neither {} nor {} are quoting tokens.",
                token_base.symbol,
                token_quote.symbol
            );
        }
    }
    Ok(())
}

async fn update_token_price(token: &TokenEntity, new_price: f64, timestamp: DateTime<Utc>) -> Result<(), DbErr> {
    let mut token_update = token.clone();
    token_update.updated_at = Utc::now();
    token_update.price = new_price;
    log::info!("Updating token price: {} - {}", token.symbol, new_price);
    TokenService::update(token_update).await?;
    let price_data = PriceDataEntity {
        id: Uuid::new_v4(),
        token_id: token.id,
        price: new_price,
        timestamp,
    };
    log::info!("Updating price data: {}", price_data.token_id);
    PriceDataService::create(price_data).await?;
    Ok(())
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
        Err(_err)=>{
            None
        }
    }


}

async fn refresh_mira_pool(mira_pool: MiraPoolsEntity) -> MiraPoolsEntity{
    get_mira_pool_metadata(mira_pool).await
}

async fn get_block_time_by_block_height(provider: &Provider, block_height: u32) -> fuels::prelude::Result<DateTime<Utc>> {
    log::info!("Fetching block time by block height: {}",block_height);
    let block = provider.block_by_height(BlockHeight::new(block_height.clone())).await?;
    Ok(block.unwrap().header.time.unwrap())
}

async fn get_start_block_number() ->u32 {
    let mut block_number: u32;
    match SyncStatusService::get_status().await {
        Ok(Some(sync_status_entity)) => {
            block_number = sync_status_entity.block_number as u32 +1
        },
        Ok(None) => { block_number = 0;},
        Err(_) => { block_number = 0; },//TODO - Exception management
    }
    if block_number <= 1{
        block_number = CONFIG.default.tx_log_start_block_number.clone() as u32;
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
    let window_range_hours = CONFIG.default.calculation_window.clone() as i64;

    // Calculate the cutoff time
    let cutoff_time = Utc::now() - Duration::from_mins(window_range_hours as u64);

    // Check if the block_time is within the calculation window
    block_time >= cutoff_time
}

async fn get_token_details_by_asset_id(provider: &Provider,asset_id: &AssetId) -> Result<Option<TokenEntity>, Error>{

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


        let bench_contract = Bech32ContractId
        ::from(ContractId::from_str(CONFIG.default.cdi_fuel_token_gateway_dependency.as_str())
            .unwrap_or(ContractId::zeroed()));

        let response = fuel_token_gateway.methods().name(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
        ]).simulate(Execution::StateReadOnly).await;
        //log::info!("TOKEN FROM GATEWAY {:?}",response);

        match response{
            Ok(call_response) => {
                match call_response.value {
                    Some(token_name) => {
                        let token_symbol = fuel_token_gateway.methods().symbol(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                        let token_decimals = fuel_token_gateway.methods().decimals(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                        let token_entity = TokenEntity{
                            id: Uuid::new_v4(),
                            address: asset_id.to_string(),
                            symbol: token_symbol,
                            name: token_name,
                            price: 0.0,
                            volume_24: 0.0,
                            decimals: token_decimals as i32,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            quoting: false,
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
            Err(_e) => {
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

async fn get_mira_token_details_by_asset_id(provider: &Provider,asset_id: &AssetId) -> Result<Option<TokenEntity>,Error>{

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
        .simulate(Execution::StateReadOnly).await;

        match response{
            Ok(call_response) => {
                match call_response.value {
                    Some(token_name) => {
                        let token_symbol = mira_contract.methods().symbol(asset_id.clone()).simulate(Execution::StateReadOnly).await?.value.unwrap();
                        let token_decimals = mira_contract.methods().decimals(asset_id.clone()).simulate(Execution::StateReadOnly).await?.value.unwrap();

                        let token_entity = TokenEntity{
                            id: Uuid::new_v4(),
                            address: asset_id.to_string(),
                            symbol: token_symbol,
                            name: token_name,
                            price: 0.0,
                            volume_24: 0.0,
                            decimals: token_decimals as i32,
                            created_at: Utc::now(),
                            updated_at: Utc::now(),
                            quoting: false,
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

                                        let token_symbol = fuel_gateway.methods().symbol(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
                                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();
                                        let token_decimals = fuel_gateway.methods().decimals(asset_id.clone()).with_contract_ids(&[bench_contract.clone(),
                                        ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                                        let token_entity = TokenEntity {
                                            id: Uuid::new_v4(),
                                            address: asset_id.to_string(),
                                            symbol: token_symbol,
                                            name: token_name,
                                            price: 0.0,
                                            volume_24: 0.0,
                                            decimals: token_decimals as i32,
                                            created_at: Utc::now(),
                                            updated_at: Utc::now(),
                                            quoting: false,
                                        };
                                        log::info!("Fuel Gateway - All data ready to create new Token entity: {:?}",token_entity);
                                        Ok(Some(TokenService::create(token_entity).await.unwrap()))
                                    },
                                    None => {
                                        log::info!("No token found in fuel gateway");
                                        let unknown_token = UnknownTokenEntity {
                                            id: Uuid::new_v4(),
                                            address: asset_id.to_string(),
                                        };
                                        let _ = UnknownTokenService::create_if_not_exists(unknown_token).await;
                                        Ok(None)
                                    }
                                }
                            },
                            Err(_e)=>{
                                log::info!("Fuel - No asset found - ext");
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
            Err(_e) => {
                log::info!("Mira - No asset found - ext");
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
            if let Ok(provider) = Provider::connect(CONFIG.default.rpc_url_one.as_str()).await {
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
