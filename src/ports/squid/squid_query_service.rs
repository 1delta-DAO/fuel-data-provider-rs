use std::collections::HashMap;
use std::error::Error;
use std::io::Read;
use fuel_tx::policies::Policies;
use fuel_tx::{Script, Transaction};
use fuel_tx::field::ChargeableBody;
use fuel_types::canonical::Deserialize;
use reqwest::Client;
use serde_json::{json, Value};

const SQUID_ENDPOINT: &str = "https://v2.archive.subsquid.io/network/fuel-mainnet";

pub struct SquidQueryService;

impl SquidQueryService {

    /*

    History is available for blocks lower for around 40000 from the current highest

     */

    pub async fn get_worker_url(block_number: u32) -> Result<String, Box<dyn Error>> {
        let url = format!("{}/{}/worker", SQUID_ENDPOINT, block_number);
        let response = Client::new().get(&url).send().await?.text().await?;
        log::info!("ANS: {:?}", response);
        Ok(response)
    }

    pub async fn get_logs_by_block_number(block_number: u32) -> Result<Value, Box<dyn Error>> {
        let worker_url = SquidQueryService::get_worker_url(block_number).await?;
        let request_body = json!({
            "type": "fuel",
            "fromBlock": block_number,
            "toBlock": block_number+10,
            "fields": {
                "receipt": {"contract": true, "receiptType": true, "toAddress": true, "amount": true,"val":true, "assetId": true, "data":true},
                /*"transaction": {
                            //"index": true,
                            "hash": true,
                            "type": true,
                            "status": true,
                            "isScript": true,
                            //"isCreate": true,
                            "inputAssetIds": true,
                            "inputContracts": true,
                            //"witnesses": true,
                            //"receiptsRoot":true,
                            "script": true,
                            "scriptData": true,
                            //"salt": true,
                            //"storageSlots": true,
                            "rawPayload": true,
                            //"bytecodeWitnessIndex": true,
                            //"bytecodeRoot": true,
                            //"subsectionIndex": true,
                            //"subsectionsNumber": true,
                            //"proofSet": true,
                            //"upgradePurpose": true,
/*                            "inputs": {
                                "index": true,
                                "transactionIndex": true,
                                "type": true,
                                "owner": true,
                                "amount": true,
                                "assetId": true
                            },
                            "outputs": {
                                "index": true,
                                "transactionIndex": true,
                                "type": true,
                                "recipient": true,
                                "amount": true
                            }*/
                        },*/
            },
            //"transactions": [{"type":["Script"],"receipts":true, "inputs":false, "outputs":true}],
            "receipts": [{"type": ["LOG_DATA"],"transaction":true}]
        });

        let response = Client::new()
            .post(&worker_url)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await?;

        let response_json: Value = response.json().await?;
        log::info!("ANS: {}", serde_json::to_string_pretty(&response_json)?);

        let mut block_map: HashMap<u32, Vec<String>> = HashMap::new();

        if let Some(blocks) = response_json.as_array() {
            for block in blocks {
                if let Some(block_number) = block.get("header").and_then(|h| h.get("number")).and_then(|n| n.as_u64()) {
                    let block_number = block_number as u32; // Konwersja na u32
                    //log::info!("Processing block: {}", block_number);

                    if let Some(transactions) = block.get("transactions").and_then(|t| t.as_array()) {
                        for tx in transactions {
                            if let Some(is_script) = tx.get("isScript").and_then(|t| t.as_str()) {
                                log::info!("Is Script: {}",is_script);
                                if let Some(raw_payload) = tx.get("rawPayload").and_then(|r| r.as_str()) {
                                    log::info!("Tx RawPayload for block {}: {}", block_number, raw_payload);

                                    // Dodajemy rawPayload do odpowiedniego bloku w mapie
                                    block_map.entry(block_number)
                                        .or_insert_with(Vec::new)
                                        .push(raw_payload.to_string());
                                    //decode_fuel_transaction(&raw_payload);
                                    //decode_script_transaction(&raw_payload).unwrap();
                                }

                            }
                        }
                    }
                }
            }
        }

        //log::info!("Final Block Map: {:?}", block_map);

        Ok(response_json)
    }
}

pub fn decode_fuel_transaction(raw_payload: &str) -> Result<(), Box<dyn Error>> {
    let bytes = hex::decode(raw_payload.trim_start_matches("0x")).unwrap();
    log::info!("Decoded raw payload: {}", bytes.len());

    let transaction: Transaction = bincode::deserialize(&bytes).unwrap();

    log::info!("Decoded transaction: {:#?}", transaction);

    //let json_str = String::from_utf8(bytes).unwrap();
    //log::info!("Decoded json: {}", json_str);

    //let tx: Transaction = serde_json::from_str(&json_str)?;

    //log::info!("Decoded tx: {:#?}", tx);

    Ok(())
}

fn decode_script_transaction(raw_payload: &str) -> Result<Script, DecodingError> {
    let bytes = hex::decode(raw_payload.trim_start_matches("0x")).unwrap();

    log::info!("Bytes length: {}", bytes.len());

    let mut buffer = &bytes[..];

    let mut discriminant_buffer = [0u8; 8];
    buffer.read_exact(&mut discriminant_buffer).unwrap();
    log::info!("Discriminant bytes: {:?}", discriminant_buffer);

    let mut transaction = Transaction::decode_static(&mut &bytes[..]).unwrap();

    transaction.decode_dynamic(&mut &bytes[..]).unwrap();

    match transaction {
        Transaction::Script(script) => {
            log::info!("Decoded script: {:#?}", script);
            Ok(script)
        },
        _ => Err(DecodingError::InvalidTransactionType) // lub własny typ błędu
    }
}
#[derive(Debug)]
pub enum DecodingError {
    InvalidTransactionType,
    DeserializationError(fuel_types::canonical::Error)
}