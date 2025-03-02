use avail_rust::prelude::*;
use dotenvy::dotenv;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // // if let Err(e) = create_application_key().await {
    // //     eprintln!("Error: {:?}", e);
    // // }

    // // Example of reading a binary file
    // // Uncomment and modify the path to test
    // // if let Err(e) = read_and_submit_binary_file("path/to/your/file.bin").await {
    // //     eprintln!("Error reading and submitting binary file: {:?}", e);
    // // }

    // if let Err(e) = submit_data().await {
    //     eprintln!("Error: {:?}", e);
    // }

    // // if let Err(e) = read_data_by_hash().await {
    // //     eprintln!("Error: {:?}", e);
    // // }

    let watch_dir = "data"; // Directory to monitor
    let mut known_files = HashSet::new();

    // Create directory if it doesn't exist
    if !Path::new(watch_dir).exists() {
        std::fs::create_dir(watch_dir).expect("Failed to create watch directory");
    }

    loop {
        // Read all files in directory
        if let Ok(entries) = std::fs::read_dir(watch_dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    // Only process .bin files that we haven't seen before
                    if path.extension().and_then(|ext| ext.to_str()) == Some("bin")
                        && !known_files.contains(&path)
                    {
                        println!("New file detected: {:?}", path);

                        if let Err(e) = submit_data().await {
                            eprintln!("Error submitting data for {:?}: {:?}", path, e);
                        } else {
                            // Add to known files after successful processing
                            known_files.insert(path);
                        }
                    }
                }
            }
        }

        // Wait before next check
        sleep(Duration::from_secs(5)).await;
    }
}

// type ApplicationKeyCreatedEvent = avail::data_availability::events::ApplicationKeyCreated;
// pub async fn create_application_key() -> Result<(), ClientError> {
//     // Create a new SDK instance
//     let sdk = SDK::new("wss://turing-rpc.avail.so/ws").await?;

//     // Loading seed phrase and creating an account derived from the seed
//     dotenv().ok();
//     let seed = env::var("SEED").expect("SEED environment variable is not set");
//     let account = account::from_secret_uri(&seed)?;
//     println!("Account Address: {}", account.public_key().to_account_id());

//     // Application Key Creation
//     // Please note that if an application key with the same `key` already exists, the transaction will fail.

//     let key = "Grid Guru Avail".as_bytes().to_vec();
//     let tx = sdk.tx.data_availability.create_application_key(key);
//     let res = tx
//         .execute_and_watch_inclusion(&account, Options::default())
//         .await?;
//     assert_eq!(
//         res.is_successful(),
//         Some(true),
//         "Transactions must be successful"
//     );

//     let events = res.events.as_ref().unwrap();
//     let event = events.find_first::<ApplicationKeyCreatedEvent>().unwrap();
//     let Some(event) = event else {
//         return Err("Failed to get Application Key Created Event".into());
//     };
//     let app_id = event.id.0;
//     println!("Application Key Created: {}", app_id);

//     Ok(())
// }

type DataSubmissionCall = avail::data_availability::calls::types::SubmitData;
pub async fn submit_data() -> Result<(), ClientError> {
    let sdk = SDK::new("wss://turing-rpc.avail.so/ws").await?;

    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable is not set");
    let account = account::from_secret_uri(&seed)?;
    println!("Account Address: {}", account.public_key().to_account_id());

    // Please note that the tx will fail if this application key does not exist
    let my_application_key = env::var("APP_ID").expect("APP_ID is not correct");
    let my_application_key = my_application_key.parse::<u32>().unwrap();

    // Data Submission
    let data = String::from("My Data").into_bytes();
    let options = Options::new().app_id(my_application_key);
    let tx: Transaction<avail_rust::transactions::DataAvailabilityCalls::SubmitData> =
        sdk.tx.data_availability.submit_data(data);
    let res: TransactionDetails = tx.execute_and_watch_inclusion(&account, options).await?;
    assert_eq!(
        res.is_successful(),
        Some(true),
        "Transactions must be successful"
    );

    println!(
        "Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
        res.block_hash, res.block_number, res.tx_hash, res.tx_index
    );

    // Decoding
    let decoded = res.decode_as::<DataSubmissionCall>().await?;
    let Some(decoded) = decoded else {
        return Err("Failed to get Data Submission Call data".into());
    };

    let data = to_ascii(decoded.data.0).unwrap();
    println!("Call data: {:?}", data);

    println!("Data Submission finished correctly");

    Ok(())
}

pub async fn read_data_by_hash() -> Result<(), ClientError> {
    // Create a new SDK instance
    let sdk = SDK::new("wss://turing-rpc.avail.so/ws").await?;

    let block_hash =
        new_h256_from_hex("0xfa13225d52499b69a723d59b24c5d6627ee31551d84069747bb14211dbc55fa0")?;

    let block = Block::new(&sdk.client, block_hash).await?;

    // All Block Blobs by Hash
    let tx_hash =
        new_h256_from_hex("0x3084452610b67ec9d9855439a780caa27ec30c7a0809d106dbf51c90fa8495a3")?;

    let blobs = block.data_submissions(Filter::new().tx_hash(tx_hash));
    assert_eq!(blobs.len(), 1, "");

    let blob = &blobs[0];

    // Printout All Block Blobs by Hash
    let blob_data = blob.to_ascii().unwrap();
    assert_eq!(blob.tx_hash, tx_hash, "Tx Hash must be the same");

    println!(
        "Tx Hash: {:?}, Tx Index: {}, Data: {:?}, App Id: {}, Tx Singer: {:?}",
        blob.tx_hash,
        blob.tx_index,
        blob_data,
        blob.app_id,
        blob.ss58address(),
    );

    Ok(())
}

/// Reads binary data from a file and returns it as a Vec<u8>
///
/// # Arguments
///
/// * `file_path` - Path to the binary file to read
///
/// # Returns
///
/// * `Result<Vec<u8>, io::Error>` - The binary data as a vector of bytes or an error
///
/// # Example
///
/// ```
/// let binary_data = read_binary_file("path/to/file.bin")?;
/// ```
pub fn read_binary_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<u8>, io::Error> {
    // Open the file in read-only mode
    let mut file = File::open(file_path)?;

    // Create a vector to store the file contents
    let mut buffer = Vec::new();

    // Read the whole file into the buffer
    file.read_to_end(&mut buffer)?;

    // Return the buffer
    Ok(buffer)
}

pub async fn submit_data_from_bin(file_path: &str) -> Result<(), ClientError> {
    let sdk = SDK::new("wss://turing-rpc.avail.so/ws").await?;

    dotenv().ok();
    let seed = env::var("SEED").expect("SEED environment variable is not set");
    let account = account::from_secret_uri(&seed)?;
    println!("Account Address: {}", account.public_key().to_account_id());

    // Please note that the tx will fail if this application key does not exist
    let my_application_key = env::var("APP_ID").expect("APP_ID is not correct");
    let my_application_key = my_application_key.parse::<u32>().unwrap();

    // Read binary data from file
    let data = read_binary_file(file_path).unwrap();

    // Data Submission
    let options = Options::new().app_id(my_application_key);
    let tx: Transaction<avail_rust::transactions::DataAvailabilityCalls::SubmitData> =
        sdk.tx.data_availability.submit_data(data);
    let res: TransactionDetails = tx.execute_and_watch_inclusion(&account, options).await?;
    assert_eq!(
        res.is_successful(),
        Some(true),
        "Transactions must be successful"
    );

    println!(
        "Block Hash: {:?}, Block Number: {}, Tx Hash: {:?}, Tx Index: {}",
        res.block_hash, res.block_number, res.tx_hash, res.tx_index
    );

    // Decoding
    let decoded = res.decode_as::<DataSubmissionCall>().await?;
    let Some(decoded) = decoded else {
        return Err("Failed to get Data Submission Call data".into());
    };

    let data = to_ascii(decoded.data.0).unwrap();
    println!("Call data: {:?}", data);

    println!("Data Submission finished correctly");

    Ok(())
}
