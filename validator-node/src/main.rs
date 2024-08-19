use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // Connect to the local validator
    let client = RpcClient::new_with_commitment("http://localhost:8899".to_string(), CommitmentConfig::finalized());

    println!("Listening for new blocks...");

    let mut previous_slot = client.get_slot().expect("Failed to get the previous slot");

    loop {
        let current_slot = client.get_slot().expect("Failed to get the current slot");

        if current_slot > previous_slot {
            for slot in previous_slot + 1..=current_slot {
                 match client.get_block(slot) {
                    Ok(block) => {
                        let transactions_count = block.transactions.len();
                        println!("Block {} with block hash {} produced with {} transactions.", slot, block.blockhash, transactions_count);
                    }
                    Err(err) => {
                        println!("Failed to fetch block {} {:?}", slot,err);
                    }
                    }
                }
            previous_slot = current_slot;
        }
        sleep(Duration::from_secs(5)).await;

    }
}
