use std::str::FromStr;
use fuels::prelude::*;
use std::time::Duration;
use fuels::tx::TxParameters;
use fuels::types::BlockHeight;
use fuels::types::output::Output;
use serde::Deserialize;

pub struct TxMonitor;

abigen!(Contract(
   name = "FuelTokenGateway",
    abi = "resources/abi/fuel_token_gateway/out/debug/bridge_fungible_token-abi.json"
),);

impl TxMonitor{

    pub async fn monitor_transactions() -> Result<()> {
        let provider = Provider::connect("https://mainnet.fuel.network").await?;
        let mut wallet = WalletUnlocked::new_random(None);
        wallet.set_provider(provider.clone());

        let balances = wallet.get_balances().await?;
        println!("Balances: {:?}", balances);

        let contract_id = ContractId::from_str("0x4ea6ccef1215d9479f1024dff70fc055ca538215d2c8c348beddffd54583d0e8")?;
        let fuel_token_gateway = FuelTokenGateway::new(contract_id,wallet);

        let mut last_block = provider.latest_block_height().await?;

        println!("Starting from block: {}", last_block);

        loop {
            let current_block = provider.latest_block_height().await?;

            if current_block > last_block {
                for block_height in last_block..=current_block {
                    let block = provider.block_by_height(BlockHeight::from(block_height)).await?;

                    if let Some(block) = block {
                        println!("\nNew blok: {}", block_height);

                        for tx in block.transactions {
                            let txr = provider.get_transaction_by_id(&tx).await?.unwrap();
                            //println!("{:?}",txr);
                            // Uzyskanie dostępu do właściwego pola `transaction`
                            // Dostęp do transakcji jako TransactionType
                            let transaction = txr.transaction;
                            //println!("TRANS: {:?}",transaction);
                            match transaction{
                                TransactionType::Mint(mint_tx) =>{
                                    println!("MINT");
                                    let asset_id = mint_tx.mint_asset_id();
                                    println!("{:?}",mint_tx.mint_asset_id());
                                    //let token_details = fetch_token_details(&asset_id.to_string()).await.unwrap();
                                    //log::info!("Mint {:?}",token_details);

                                    //let token_name
                                    //    = fuel_token_gateway.methods().name(asset_id.clone()).simulate(Default::default()).await;
                                    //println!("MINT TN: {:?}",token_name);
                                },
                                TransactionType::Script(script_tx) => {
                                    println!("SCRIPT");
                                    for output in script_tx.outputs() {
                                        if let Some(asset_id) = extract_asset_id(output) {
                                            println!("Script transaction asset_id: {}", asset_id);
                                            let token_name
                                                = fuel_token_gateway.methods().name(asset_id.clone()).call().await;
                                            println!("SCRIPT TN: {:?}",token_name);
                                            //let token_details = fetch_token_details(&asset_id).await?;
                                            //log::info!("Script {:?}",token_details);
                                        }
                                    }
                                },
                                TransactionType::Create(create_tx) => {
                                    println!("CREATE");
                                    // Iteracja po wyjściach z `CreateTransaction`
                                    for output in create_tx.outputs() {
                                        if let Some(asset_id) = extract_asset_id(output) {
                                            //println!("Create transaction asset_id: {}", asset_id);
                                            //let token_details = fetch_token_details(&asset_id).await?;
                                            //log::info!("Create {:?}",token_details);
                                            let token_name
                                                = fuel_token_gateway.methods().name(asset_id.clone()).simulate(Default::default()).await;
                                            println!("CREATE TN: {:?}",token_name);
                                        }
                                    }
                                },

                                _ => {
                                    println!("Other TX type");
                                }
                            }
                        }
                    }
                }

                last_block = current_block;
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

async fn fetch_token_details(asset_id: &str) -> Result<TokenDetails> {

    let provider = Provider::connect("https://mainnet.fuel.network").await?;


    let url = format!("https://api.fuel.network/assets/{}", asset_id);
    let response = reqwest::get(&url).await.unwrap();
    let token_details = response.json::<TokenDetails>().await.unwrap();
    Ok(token_details)
}

#[derive(Debug, Deserialize)]
struct TokenDetails {
    name: String,
    symbol: String,
    decimals: u8,
}