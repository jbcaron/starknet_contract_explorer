mod contract;
mod db;
mod history;
mod request;
mod state_update;

use db::Database;
use state_update::{DeclaredClass, StateUpdate, StorageDiff};

const FEEDER_GATEWAY: &str = "https://alpha-mainnet.starknet.io/feeder_gateway";
//const FEEDER_GATEWAY: &str = "http://127.0.0.1:3000/feeder_gateway";

const START_BLOCK: u64 = 0;
const END_BLOCK: u64 = 10_000;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    log::info!("🚀 Starting Starknet Explorer 🚀");
    let db = std::sync::Arc::new(Database::new("db").unwrap());
    log::info!("💾 Database created");

    log::info!("🔄 Syncing from block {} to {}", START_BLOCK, END_BLOCK);
    match sync(db.clone(), START_BLOCK, END_BLOCK).await {
        Ok(_) => log::info!("🚀 Synced"),
        Err(e) => log::error!("❌ Sync error: {e}"),
    }

    println!("🚀 Welcome to the Starknet CLI Explorer 🚀");
    loop {
        match request::prompt(&db) {
            Ok(true) => break,
            Ok(false) => continue,
            Err(e) => println!("❌ Error: {e}"),
        }
    }
    log::info!("Exiting");
    drop(db);
    Ok(())
}

const SYMULTANEOUS_REQUESTS: usize = 20;

async fn sync(
    db: std::sync::Arc<Database>,
    start_block: u64,
    end_block: u64,
) -> Result<(), String> {
    // Instantiate the client (could be reused for multiple requests)
    let client = std::sync::Arc::new(reqwest::Client::new());

    for block_number in (start_block..=end_block).step_by(SYMULTANEOUS_REQUESTS) {
        log::info!(
            "Processing block: {} to {}",
            block_number,
            block_number + SYMULTANEOUS_REQUESTS as u64
        );
        let block_number = block_number..block_number + SYMULTANEOUS_REQUESTS as u64;
        let time = std::time::Instant::now();
        let fetches = block_number
            .map(|number| (fetch_and_deserialize(number, client.clone())))
            .collect::<Vec<_>>();

        let results = futures::future::join_all(fetches).await;
        log::info!("Fetched and deserialized in {:?}", time.elapsed());
        let time = std::time::Instant::now();

        for result in results {
            let (block_number, state_update) = result.map_err(|e| format!("fetch error: {e}"))?;
            let mut handles_deployed_contract = Vec::new();
            let mut handles_replaced_contract = Vec::new();
            let mut handles_nonce = Vec::new();
            let mut handles_key = Vec::new();
            // insert new classes into the class tree
            for DeclaredClass {
                class_hash,
                compiled_class_hash,
            } in state_update.state_diff.declared_classes
            {}

            // insert new contracts into the contract tree
            for deployed_contract in state_update.state_diff.deployed_contracts {
                let db_clone = db.clone();
                let handle = tokio::spawn(async move {
                    let _ = db_clone
                        .insert_class_hash(
                            deployed_contract.address,
                            deployed_contract.class_hash,
                            block_number,
                        )
                        .map_err(|e| log::error!("insert class hash error: {e}"));
                });
                handles_deployed_contract.push(handle);
            }

            // update the contracts class hash in the contract tree
            for replaced_contract in state_update.state_diff.replaced_classes {
                let db_clone = db.clone();
                let handle = tokio::spawn(async move {
                    let _ = db_clone
                        .insert_class_hash(
                            replaced_contract.address,
                            replaced_contract.class_hash,
                            block_number,
                        )
                        .map_err(|e| log::error!("insert class hash error: {e}"));
                });
                handles_replaced_contract.push(handle);
            }

            // update nonces into the contract tree
            for (contract_address, nonce) in state_update.state_diff.nonces {
                let db_clone = db.clone();
                let handle = tokio::spawn(async move {
                    let _ = db_clone
                        .insert_nonce(contract_address, nonce, block_number)
                        .map_err(|e| log::error!("insert nonce error: {e}"));
                });
                handles_nonce.push(handle);
            }

            // insert key-value pairs into the contract tree
            for (contract_address, storage_diffs) in state_update.state_diff.storage_diffs {
                for StorageDiff { key, value } in storage_diffs {
                    let db_clone = db.clone();
                    let handle = tokio::spawn(async move {
                        let _ = db_clone
                            .insert_key(contract_address, key, value, block_number)
                            .map_err(|e| log::error!("insert key error: {e}"));
                    });
                    handles_key.push(handle);
                }
            }

            tokio::join!(
                futures::future::join_all(handles_deployed_contract),
                futures::future::join_all(handles_replaced_contract),
                futures::future::join_all(handles_nonce),
                futures::future::join_all(handles_key)
            );
        }
        log::info!("Processed blocks in {:?}", time.elapsed());
    }
    Ok(())
}

const MAX_ATTEMPS: u32 = 20;

async fn fetch_and_deserialize(
    block_number: u64,
    client: std::sync::Arc<reqwest::Client>,
) -> Result<(u64, StateUpdate), String> {
    let client = client.as_ref();

    let url = format!(
        "{}/get_state_update?blockNumber={}",
        FEEDER_GATEWAY, block_number
    );

    let mut attempts = 0;

    let mut response;

    while attempts <= MAX_ATTEMPS {
        response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("send fail: {e}"))?;

        let status = response.status();

        match status {
            reqwest::StatusCode::OK => {
                // Deserialize the JSON into a Rust struct
                let state_update = response
                    .json::<StateUpdate>()
                    .await
                    .map_err(|e| format!("serialisation: {e}"))?;

                return Ok((block_number, state_update));
            }
            reqwest::StatusCode::TOO_MANY_REQUESTS => {
                log::info!("Too many requests, waiting...");
                let wait_time = 2u64.pow(attempts) * 1000; // expo
                tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
                attempts += 1;
                continue;
            }
            _ => {
                log::info!("code status: {status}, waiting...");
                let wait_time = 2u64.pow(attempts) * 1000; // expo
                tokio::time::sleep(tokio::time::Duration::from_millis(wait_time)).await;
                attempts += 1;
                continue;
            } //_ => return Err(format!("code status: {status}")),
        }
    }
    Err("max attempts".into())
}
