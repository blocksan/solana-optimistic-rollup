use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::hashv, instruction::{AccountMeta, Instruction}, signature::{read_keypair_file, Keypair, Signer}, transaction:: Transaction
};
// pub fn fetch_block(slot: limit:) -> {

// }

const  SOLANA_ROLLUP_CONTRACT:&str = "FozZUVREfj6jYfvyTh4qb5nB8dPkysF54xSexACcT8jc";
fn main() {
    let rpc_url = "http://localhost:8899";  // Assuming your test validator is running on the default RPC port
    let client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let payer = read_keypair_file("/Users/sandeepghosh/config/solana/optimistic-rollup/localhost/user-dg-localhost-key.json").unwrap();

    // Fetch the latest block
    let slot = client.get_slot().unwrap();
    println!("Latest slot: {}", slot);

    let start_slot:u64 = 90525;
    let limit: usize = 10;
    let mut block_numbers = vec![0u64;10];
    let mut block_hashes = Vec::with_capacity(10);

    // Fetch block data for a specific slot
    match client.get_blocks_with_limit(start_slot, limit) {
        Ok(blocks) => {
            block_numbers = blocks;
        },
        Err(e) => eprintln!("Error fetching block: {}", e),
    }

    let block_iterator = block_numbers.into_iter();
    for block in block_iterator {
        println!("Found block {}", block);
        match client.get_block(block){
            Ok(block_details) =>{
                // println!("Block prevhash {}", block_details.previous_blockhash);
                // println!("Block hash {}", block_details.blockhash);
                block_hashes.push(block_details.blockhash);
            },
            Err(e) => eprintln!("Block not found: {}",e)
        }
    }

    let block_hash_iterator = block_hashes.clone().into_iter();
    for block_hash in block_hash_iterator {
        println!("Block hash => {}", block_hash);
    }

    let merkel_root_hash = build_merkel_tree(block_hashes).unwrap();
    
    let root_hash_commit_instruction = create_rollup_root_hash_commit_instruction(&merkel_root_hash, &payer.pubkey());
    
    let root_hash_commit_transaction = prepare_solana_transaction(&client, &payer, vec![root_hash_commit_instruction]).unwrap();

    println!("Sending merkel root hash to the Rollup Contract {}",merkel_root_hash);
    let transaction_signature = send_transaction_to_solana(&client, &root_hash_commit_transaction).unwrap();

    println!("Transaction successfully published {}", transaction_signature);
}

fn build_merkel_tree(hashes: Vec<String>) -> Option<String>{
    if hashes.len() == 0 {
        None
    }else if hashes.len() == 1 {
        Some(hashes[0].clone())
   }else{
    let mut hashes_clone = hashes.clone();
    while hashes_clone.len() > 1 {
        let mut next_level: Vec<String> = Vec::new();

        for i in (0..hashes_clone.len()).step_by(2) {
            let left = &hashes_clone[i];
            let right = if i + 1 < hashes_clone.len() { 
                &hashes_clone[i + 1] 
            } else { 
                &hashes_clone[i] 
            };
            let parent_hash = hashv(&[left.as_ref(), right.as_ref()]).to_string();
            next_level.push(parent_hash);
        }
        hashes_clone = next_level;
    }
    Some(hashes_clone[0].clone())
   }
}


pub fn prepare_solana_transaction(
    client: &RpcClient,
    payer: &Keypair,
    instruction: Vec<Instruction>,
) -> Option<Transaction>{
    let mut transaction: Transaction = Transaction::new_with_payer(
        &instruction, 
        Some(&payer.pubkey()), // Payer for the transaction
    );
    let recent_block_hash = client.get_latest_blockhash().ok().unwrap();
    println!("Recent blockhash found {}",recent_block_hash.to_string().clone());
    transaction.message.recent_blockhash = recent_block_hash;
    transaction.sign(&[payer], recent_block_hash); //signing the transaction
    Some(transaction)
}

fn create_rollup_root_hash_commit_instruction(root_hash: &str, owner_public_key: &Pubkey) -> Instruction{
    let program_pubkey_result: Result<Pubkey, solana_sdk::pubkey::ParsePubkeyError> = Pubkey::from_str(SOLANA_ROLLUP_CONTRACT);
    let program_id = program_pubkey_result.unwrap();

    let derived_account = Pubkey::create_with_seed(owner_public_key, "sequencer", &program_id).unwrap();

    let accounts = vec![
        AccountMeta::new(derived_account, false), // The account that will be updated
    ];
    // let data = root_hash;
    let data = root_hash.as_bytes();
    Instruction::new_with_bytes(program_id, data, accounts)
}

fn send_transaction_to_solana(
    client: &RpcClient,
    transaction: &Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    // Send and confirm the transaction
    let signature = client.send_and_confirm_transaction(transaction)?;

    Ok(signature.to_string())
}


// { previous_blockhash: "HVinyA3wjQYfuwXhky7iJhg8gtGbPuXck76YFzt7ih85", blockhash: "Bx8jC1zwU6tXjCLxFwYok57KmG4nxkEar8LwX5soeKma", parent_slot: 90524, transactions: [EncodedTransactionWithStatusMeta { transaction: Json(UiTransaction { signatures: ["58xPN4ucwXeMTHDdhZttUaPPHhuWCHzViCeypT1YhDfFjvJjZP6mAcya8du2c12VUjh1awsoJM6goQNELjLQVwpQ", "5Wy2ffn6ZtVg7EGofmFsQTgSnFmURMLKBY3EWFqLTpoz27s2u7K564daxrrGVRhPrnGJzRYw1s6RKqWnAdK5bv2D"], message: Raw(UiRawMessage { header: MessageHeader { num_required_signatures: 2, num_readonly_signed_accounts: 0, num_readonly_unsigned_accounts: 1 }, account_keys: ["4Yty5RDZfdhGd5XZVqRVSem1VzRaDD2Hm5DfxQuHYJKP", "HHbmDLoShMwuDD7onF7xGgVN5MMcwqJBdsH5GjJBky1y", "Vote111111111111111111111111111111111111111"], recent_blockhash: "HVinyA3wjQYfuwXhky7iJhg8gtGbPuXck76YFzt7ih85", instructions: [UiCompiledInstruction { program_id_index: 2, accounts: [1, 1], data: "Fk63Pj2wga9EkJUhjcFrWFkyJih2q4K63ffjvBwDoAhNv5ggiYUotqRXxY3LJxTEUPcopV7BnyvBwWZXMVop6PBcMGAmgh5N4igE5n6NPWEv8wKr3YmnsECmbLupiscYxUEbBMNJ5Uj7uYhVZjneq6LW4h5MZq", stack_height: None }], address_table_lookups: None }) }), meta: Some(UiTransactionStatusMeta { err: None, status: Ok(()), fee: 10000, pre_balances: [499547520000, 1000000000000000, 1], post_balances: [499547510000, 1000000000000000, 1], inner_instructions: Some([]), log_messages: Some(["Program Vote111111111111111111111111111111111111111 invoke [1]", "Program Vote111111111111111111111111111111111111111 success"]), pre_token_balances: Some([]), post_token_balances: Some([]), rewards: Some([]), loaded_addresses: Some(UiLoadedAddresses { writable: [], readonly: [] }), return_data: Skip, compute_units_consumed: Some(2100) }), version: None }], rewards: [Reward { pubkey: "4Yty5RDZfdhGd5XZVqRVSem1VzRaDD2Hm5DfxQuHYJKP", lamports: 5000, post_balance: 499547515000, reward_type: Some(Fee), commission: None }], block_time: Some(1723801459), block_height: Some(90524) }