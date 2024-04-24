use dialoguer::{Input, Select};
use starknet_api::hash::StarkFelt;

use crate::db::Database;

#[derive(Debug, Default)]
struct Request {
    request_type: String,
    contract: Option<StarkFelt>,
    key: Option<StarkFelt>,
    block: Option<u64>,
}

pub fn prompt(db: &Database) -> Result<bool, String> {
    let request_types = [
        "class_hash",
        "nonce",
        "storage_key",
        "revert",
        "flush_db",
        "quit",
    ];
    let selection = Select::new()
        .with_prompt("Select request type")
        .default(0)
        .items(&request_types[..])
        .interact()
        .map_err(|_| "Invalid selection")?;

    let mut request = Request::default();

    match request_types[selection] {
        "class_hash" => {
            let contract = Input::<String>::new()
                .with_prompt("Enter contract address")
                .interact_text()
                .map_err(|_| "Invalid contract address")?;
            request.contract = Some(
                StarkFelt::try_from(contract.as_str()).map_err(|_| "Invalid contract address")?,
            );

            let index = Input::<u64>::new()
                .with_prompt("Enter block number")
                .interact_text()
                .map_err(|_| "Invalid block number")?;
            request.block = Some(index);

            let time = std::time::Instant::now();
            let class_hash = db
                .get_class_hash_at(request.contract.unwrap(), request.block.unwrap())
                .unwrap();
            log::info!("⏳ Processed request in {:?}", time.elapsed());
            match class_hash {
                Some(class_hash) => println!("Class hash: {}", class_hash),
                None => println!("🤷‍♂️ Class hash not found"),
            }
        }
        "nonce" => {
            let contract = Input::<String>::new()
                .with_prompt("Enter contract address")
                .interact_text()
                .map_err(|_| "Invalid contract address")?;
            request.contract = Some(
                StarkFelt::try_from(contract.as_str()).map_err(|_| "Invalid contract address")?,
            );

            let index = Input::<u64>::new()
                .with_prompt("Enter block number")
                .interact_text()
                .map_err(|_| "Invalid block number")?;
            request.block = Some(index);

            let time = std::time::Instant::now();
            let nonce = db
                .get_nonce_at(request.contract.unwrap(), request.block.unwrap())
                .unwrap();
            println!("⏳ Processed request in {:?}", time.elapsed());
            match nonce {
                Some(nonce) => println!("Nonce: {}", nonce),
                None => println!("🤷‍♂️ Nonce not found"),
            }
        }
        "storage_key" => {
            let contract = Input::<String>::new()
                .with_prompt("Enter contract address")
                .interact_text()
                .map_err(|_| "Invalid contract address")?;
            request.contract = Some(
                StarkFelt::try_from(contract.as_str()).map_err(|_| "Invalid contract address")?,
            );

            let key = Input::<String>::new()
                .with_prompt("Enter key")
                .interact_text()
                .map_err(|_| "Invalid key")?;
            request.key = Some(StarkFelt::try_from(key.as_str()).map_err(|_| "Invalid key")?);

            let index = Input::<u64>::new()
                .with_prompt("Enter block number")
                .interact_text()
                .map_err(|_| "Invalid block number")?;
            request.block = Some(index);

            let time = std::time::Instant::now();
            let value = db
                .get_key_at(
                    request.contract.unwrap(),
                    request.key.unwrap(),
                    request.block.unwrap(),
                )
                .unwrap();
            log::info!("⏳ Processed request in {:?}", time.elapsed());
            match value {
                Some(value) => println!("Value: {}", value),
                None => println!("🤷‍♂️ Key not found"),
            }
        }

        "revert" => {
            let index = Input::<u64>::new()
                .with_prompt("Enter block number")
                .interact_text()
                .map_err(|_| "Invalid block number")?;
            request.block = Some(index);

            let selection = Select::new()
                .with_prompt("Are you sure you want to revert?")
                .default(0)
                .items(&["Yes", "No"])
                .interact()
                .map_err(|_| "Invalid selection")?;

            if selection == 1 {
                return Ok(false);
            }

            let time = std::time::Instant::now();
            db.revert_to(request.block.unwrap())
                .map_err(|e| format!("Database error: {:?}", e))?;
            log::info!("⏳ Processed request in {:?}", time.elapsed());
            println!("🔙 Reverted to block {}", request.block.unwrap());
        }

        "quit" => {
            return Ok(true);
        }

        "flush_db" => {
            db.flush().map_err(|e| "Database error: {e}")?;
            println!("🧹 Database flushed");
        }
        _ => unreachable!(),
    }
    Ok(false)
}
