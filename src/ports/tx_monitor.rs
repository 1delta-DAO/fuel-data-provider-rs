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

pub struct TxMonitor;

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
static MIRA_AMM_2_CID: &str = "0x2e40f2b244b98ed6b8204b3de0156c6961f98525c8162f80162fcf53eebd90e7"; //To ten
impl TxMonitor{

    pub async fn monitor_transactions() -> Result<()> {
        let provider = Provider::connect("https://mainnet.fuel.network").await?;
        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        //let balances = wallet.get_balances().await?;
        //log::info!("Balances: {:?}", balances);

/*        let contract_id = ContractId::from_str("0x4ea6ccef1215d9479f1024dff70fc055ca538215d2c8c348beddffd54583d0e8")?;
        let fuel_token_gateway = FuelTokenGateway::new(contract_id,wallet);

        let asset_id_h: AssetId = AssetId::from_str("0x1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82").unwrap_or(AssetId::zeroed());

        let token_name = fuel_token_gateway.methods().name(asset_id_h.clone()).with_contract_ids(&[
            Bech32ContractId::from(ContractId::from_str("0x0ceafc5ef55c66912e855917782a3804dc489fb9e27edfd3621ea47d2a281156").unwrap_or(ContractId::zeroed())),
        ]).simulate(Execution::StateReadOnly).await;

        log::info!("ASSET TN: {:?}",token_name);*/

/*        let asset_id_h: AssetId = AssetId::from_str("0x1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82").unwrap_or(AssetId::zeroed());

        let token_details = self::get_token_details_by_asset_id(&provider,&asset_id_h).await?;

        log::info!("ASSET TN: {:?}",token_details);*/

        //let mut last_block = provider.latest_block_height().await?;

        let mira_cid = ContractId::from_str(MIRA_AMM_2_CID).unwrap_or(ContractId::zeroed());

        let mira_contract = MiraV1Core::new(mira_cid,wallet);

        let p_asset_id_0: AssetId = AssetId::from_str("0x1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82").unwrap_or(AssetId::zeroed());
        let p_asset_id_1: AssetId = AssetId::from_str("0x286c479da40dc953bddc3bb4c453b608bba2e0ac483b077bd475174115395e6b").unwrap_or(AssetId::zeroed());

        let mira_pool_id = MiraPoolId{
            asset_id_0: p_asset_id_0,
            asset_id_1: p_asset_id_1,
            is_stable: false
        };

        // let pool_sample
        //     = mira_contract.methods().pool_metadata((p_asset_id_0,p_asset_id_1,false))
        //     .simulate(Execution::StateReadOnly).await.unwrap().value.unwrap();

        let pool_sample = get_mira_pool_metadata(&provider,mira_pool_id).await?;

        log::info!("POOL_METADATA: {:?}",pool_sample);

        //MiraV1Core::methods()

        //pool_id: (1d5d97005e41cae2187a895fd8eab0506111e0e2f3331cd3912c15c24e3c1d82, 286c479da40dc953bddc3bb4c453b608bba2e0ac483b077bd475174115395e6b, false)


        let mut start_block:u32 = 11000000;

        let start_block_time = get_block_time_by_block_height(&provider,start_block).await;

        log::info!("Start block time: {:?}",start_block_time);

        loop {
            let current_block = provider.latest_block_height().await?;

            if current_block > start_block {
                for block_height in start_block..=current_block {
                    let block = provider.block_by_height(BlockHeight::from(block_height)).await?;

                    if let Some(block) = block {
                        //log::info!("New blok: {}", block_height);

                        for tx in block.transactions {
                            let txr = provider.get_transaction_by_id(&tx).await?.unwrap();
                            //log::info!("TX: {}",tx);
                            let transaction = txr.transaction.clone();
                            let receipts = txr.status.clone().take_receipts();
                            match transaction{
                                TransactionType::Mint(mint_tx) =>{
                                    //log::info!("MINT");
                                    //let asset_id = mint_tx.mint_asset_id();
                                },
                                TransactionType::Script(script_tx) => {
                                    //log::info!("SCRIPT");
                                    let mira_contract_id = ContractId::from_str("0x2e40f2b244b98ed6b8204b3de0156c6961f98525c8162f80162fcf53eebd90e7")?;
                                    for input in script_tx.inputs() {
                                        let cid = input.contract_id();
                                        //log::info!("CID: {:?}",cid);
                                        if cid.is_some(){
                                            if mira_contract_id == cid.unwrap().clone() {

                                                log::info!("MIRA TX: ----------------------------------------------");
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
                                                            //log::info!("LogData Receipt:");
                                                            //log::info!("{:?}",receipt);
                                                            let log_id = receipt.rb().unwrap() as u64;

//                                                            let event = MiraV1Core::SwapEvent::try_from(receipt.data().unwrap());


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
                                                    //log::info!("TX REC: {:?}",receipt);
                                                }

                                                //log::info!("TX: {:?}",script_tx);
                                                //let res = ABIDecoder::try_from(script_tx.script_data());
                                                //ABIDecoder::new(DecoderConfig::default()).decode(&MiraV1Core::pa)

                                                //log::info!("DECODED: {:?}",res.unwrap());

//                                                mira_v1_core = MiraV1Core::new(mira_contract_id,wallet);
                                                //MiraV1Core::
                                                //script_tx.script_data()

                                                /*for input in script_tx.inputs() {
                                                    log::info!("S INPUT: {:?}",input);
                                                }

                                                for output in script_tx.outputs() {
                                                    log::info!("S OUTPUT: {:?}",output);
                                                }*/

                                                log::info!("MIRA - END");
                                            }
                                        }
                                    }

                                    for output in script_tx.outputs() {

                                        //log::info!("S OUTPUT: {:?}",output);
/*                                        if output.is_change(){
                                            let token_details =
                                                get_token_details_by_asset_id(&provider, output.asset_id().unwrap()).await?;
                                            log::info!("CHANGE:----");
                                            //log::info!("CID: {}",output.contract_id().unwrap());
                                            //log::info!("AMOUNT: {}",output.amount().unwrap());
                                            log::info!("TOKEN: {:?}",token_details);
                                        }*/

/*                                        if let Some(asset_id) = extract_asset_id(output) {
                                            log::info!("Script transaction asset_id: {}", asset_id);
                                            //let token_details = fetch_token_details(&asset_id).await?;
                                            //log::info!("Script {:?}",token_details);
                                        }*/
                                    }
                                },
                                TransactionType::Create(create_tx) => {
                                    log::info!("CREATE");
                                    // Iteracja po wyjściach z `CreateTransaction`
                                    for output in create_tx.outputs() {
                                        if let Some(asset_id) = extract_asset_id(output) {
                                            //log::info!("Create transaction asset_id: {}", asset_id);
                                            //let token_details = fetch_token_details(&asset_id).await?;
                                            //log::info!("Create {:?}",token_details);
                                            //let token_name
                                            //    = fuel_token_gateway.methods().name(asset_id.clone()).simulate(Default::default()).await;
                                            //log::info!("CREATE TN: {:?}",token_name);
                                        }
                                    }
                                },

                                _ => {
                                    log::info!("Other TX type");
                                }
                            }
                        }
                    }
                }

                start_block = current_block;
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    }
}

// Funkcja do wyciągania asset_id z wyjść transakcji
fn extract_asset_id(output: &Output) -> Option<&AssetId> {
    match output {
        Output::Coin { asset_id, .. } => Some(asset_id),
        _ => None,
    }
}

//Token methods

async fn get_token_details_by_asset_id(provider: &Provider,asset_id: &AssetId) -> Result<Option<TokenDetails>>{
    let mut wallet = WalletUnlocked::new_random(None);
    wallet.set_provider(provider.clone());

    let contract_id = ContractId::from_str(FUEL_TOKEN_GATEWAY_CID).unwrap_or(ContractId::zeroed());
    let fuel_token_gateway = FuelTokenGateway::new(contract_id,wallet);

    //TODO: There has to be more efficient way to take all this data at once



    let response = fuel_token_gateway.methods().name(asset_id.clone()).with_contract_ids(&[
        Bech32ContractId::from(ContractId::from_str("0x0ceafc5ef55c66912e855917782a3804dc489fb9e27edfd3621ea47d2a281156").unwrap_or(ContractId::zeroed())),
    ]).simulate(Execution::StateReadOnly).await;

    match response{
        Ok(call_response) => {
            match call_response.value {
                Some(token_name) => {
                    let token_symbol = fuel_token_gateway.methods().symbol(asset_id.clone()).with_contract_ids(&[
                        Bech32ContractId::from(ContractId::from_str("0x0ceafc5ef55c66912e855917782a3804dc489fb9e27edfd3621ea47d2a281156").unwrap_or(ContractId::zeroed())),
                    ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                    let token_decimals = fuel_token_gateway.methods().decimals(asset_id.clone()).with_contract_ids(&[
                        Bech32ContractId::from(ContractId::from_str("0x0ceafc5ef55c66912e855917782a3804dc489fb9e27edfd3621ea47d2a281156").unwrap_or(ContractId::zeroed())),
                    ]).simulate(Execution::StateReadOnly).await?.value.unwrap();

                    let token_details = TokenDetails{
                        address: asset_id.to_string(),
                        name: token_name,
                        symbol: token_symbol,
                        decimals: token_decimals,
                    };
                    Ok(Some(token_details))
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

//Block methods

async fn get_block_time_by_block_height(provider: &Provider, block_height: u32) -> Result<DateTime<Utc>>{
    let block = provider.block_by_height(BlockHeight::new(block_height.clone())).await?;
    Ok(block.unwrap().header.time.unwrap())
}


/*async fn fetch_token_details(asset_id: &str) -> Result<TokenDetails> {

    let provider = Provider::connect("https://mainnet.fuel.network").await?;


    let url = format!("https://api.fuel.network/assets/{}", asset_id);
    let response = reqwest::get(&url).await.unwrap();
    let token_details = response.json::<TokenDetails>().await.unwrap();
    Ok(token_details)
}*/

async fn get_mira_pool_metadata(provider: &Provider, pool_id: MiraPoolId) ->Result<PoolMetadata>{

    let mut wallet = WalletUnlocked::new_random(None);
    wallet.set_provider(provider.clone());


    let mira_cid = ContractId::from_str(MIRA_AMM_2_CID).unwrap_or(ContractId::zeroed());

    let mira_contract = MiraV1Core::new(mira_cid,wallet);


    let pool_sample
        = mira_contract.methods().pool_metadata((pool_id.asset_id_0,pool_id.asset_id_1,pool_id.is_stable))
        .simulate(Execution::StateReadOnly).await.unwrap().value.unwrap();

    Ok(pool_sample)

}

#[derive(Debug, Deserialize)]
struct TokenDetails {
    address: String,
    name: String,
    symbol: String,
    decimals: u8,
}

#[derive(Debug, Deserialize)]
struct MiraPoolId {
    asset_id_0: AssetId,
    asset_id_1: AssetId,
    is_stable: bool,
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