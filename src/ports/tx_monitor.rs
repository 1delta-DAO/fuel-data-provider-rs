use fuels::prelude::*;
use std::time::Duration;
use fuels::types::BlockHeight;
use fuels::types::output::Output;
use serde::Deserialize;

pub struct TxMonitor;

impl TxMonitor{

    pub async fn monitor_transactions() -> Result<()> {
        let provider = Provider::connect("https://mainnet.fuel.network").await?;
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
                                    let token_details = fetch_token_details(&asset_id.to_string()).await.unwrap();
                                    log::info!("Mint {:?}",token_details);
                                },
                                TransactionType::Script(script_tx) => {
                                    println!("SCRIPT");
                                    for output in script_tx.outputs() {
                                        if let Some(asset_id) = extract_asset_id(output) {
                                            println!("Script transaction asset_id: {}", asset_id);
                                            let token_details = fetch_token_details(&asset_id).await?;
                                            log::info!("Script {:?}",token_details);
                                        }
                                    }
                                },
                                TransactionType::Create(create_tx) => {
                                    println!("CREATE");
                                    // Iteracja po wyjściach z `CreateTransaction`
                                    for output in create_tx.outputs() {
                                        if let Some(asset_id) = extract_asset_id(output) {
                                            println!("Create transaction asset_id: {}", asset_id);
                                            let token_details = fetch_token_details(&asset_id).await?;
                                            log::info!("Create {:?}",token_details);
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
fn extract_asset_id(output: &Output) -> Option<String> {
    match output {
        Output::Coin { asset_id, .. } => Some(format!("{:?}", asset_id)),
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