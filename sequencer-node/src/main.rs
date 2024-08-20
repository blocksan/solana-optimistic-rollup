use serde::{Deserialize, Serialize};
use std::str::FromStr;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::{
    commitment_config::CommitmentConfig, hash::hashv, instruction::{AccountMeta, Instruction}, signature::{read_keypair_file, Keypair, Signer}, transaction:: Transaction
};
const  SOLANA_ROLLUP_CONTRACT:&str = "FozZUVREfj6jYfvyTh4qb5nB8dPkysF54xSexACcT8jc";
const BATCH_SIZE: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SequencerState{
    active: bool,
    processing_transaction_pool: Vec<Transaction>,
    pending_transaction_pool: Vec<Transaction>,
    processed_transaction_pool: Vec<Transaction>,
    lock_acquired: bool
}

// Sequencer Node will push the data to the DA layer as well
// prevData => Data Avalability Layer 

// State Transition Function (prevData, current Batch of Transaction) = current merkel root hash

// impl SequencerState{
//     pub fn initiate_genesis() -> SequencerState{
//         SequencerState {
//             active: false,
//             transaction_pool: [].to_vec(),
//             pending_transaction_pool: [].to_vec(),
//             lock_acquired: false
//         }
//     }

//     pub fn sequencer_state(&self) -> &SequencerState {
//         self
//     }
// }

fn main(){
    let rpc_url = "http://localhost:8899";  // Assuming your test validator is running on the default RPC port
    let _client = RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let _payer = read_keypair_file("/Users/sandeepghosh/config/solana/optimistic-rollup/localhost/user-dg-localhost-key.json").unwrap();

    let sequencer: SequencerState = SequencerState{
        active: false,
        processed_transaction_pool: [].to_vec(),
        processing_transaction_pool: [].to_vec(),
        pending_transaction_pool: [].to_vec(),
        lock_acquired: false
    };
    
    println!("Sequencer state => {:?}", sequencer);
    // let transaction = String::from("transaction");

    //receive the transaction via POST API or Websockets from the 
    //libp2p
    // receive_transaction(transaction, &mut sequencer);
    //add the transaction to the pending_transactions_pool
    //check_for_batch_size(sequencer)
    //is_sequencer_available_to_process(sequencer)
    //acquire the lock
    //fill the process_pending_transactions pool with pending_transactions
    //process each transaction
    //create merkel root hash (MRH)
    //push the MRH to the Rollup Contract
    //release the lock
}



pub fn receive_transaction(transaction: Transaction, sequencer: &mut SequencerState) -> &SequencerState{
    if sequencer.active {
        println!("Sequencer is status => {}", sequencer.active);
        sequencer.pending_transaction_pool.push(transaction);
    }else {
        println!("Please retry sequencer is not active => {}", sequencer.active);
    }
    sequencer
}

pub fn check_for_batch_size(sequencer: &SequencerState) -> bool{
    if sequencer.pending_transaction_pool.len() >= BATCH_SIZE {
        true
    }else{
        false
    }
}

pub fn is_sequencer_available_to_process(sequencer: &SequencerState) -> bool{
    if sequencer.lock_acquired {
        false
    }else{
        true
    }
}

pub fn process_pending_transactions(sequencer: &mut SequencerState, client: &RpcClient, payer: &Keypair) {
    let batch_size_reached = check_for_batch_size(sequencer);

    if batch_size_reached {
        let mut count: usize = 0;
        //form the pending transactions array
        for transaction in sequencer.pending_transaction_pool.iter_mut() {
            if count == BATCH_SIZE {
                break;
            }else{
                sequencer.processing_transaction_pool.push(transaction.clone());
                count +=1
            }
        }

        let sequencer_in_locked_state = is_sequencer_available_to_process(&sequencer);
        if sequencer_in_locked_state {
            println!("Please try after sometime...")
        }else{
        //acquire the lock
        sequencer.lock_acquired = true;
        let mut transaction_hashes :Vec<String> = vec![];

        for pending_transaction in sequencer.pending_transaction_pool.iter_mut(){
            //process each transaction
            //TODO: compress these transactions and create a hash of it
            let transaction_processed_signature = process_transaction_on_svm(client, pending_transaction).unwrap();
            transaction_hashes.push(transaction_processed_signature);
        }

        let merkel_root_hash = build_merkel_tree(transaction_hashes).unwrap();
        let root_hash_commit_instruction = create_rollup_root_hash_commit_instruction(&merkel_root_hash, &payer.pubkey());
    
        let root_hash_commit_transaction = prepare_solana_transaction(&client, &payer, vec![root_hash_commit_instruction]).unwrap();

        println!("Sending merkel root hash to the Rollup Contract {}",merkel_root_hash);
        let transaction_signature = send_transaction_to_solana(&client, &root_hash_commit_transaction).unwrap();
        println!("Transaction successfully published {}", transaction_signature);

        //release the lock
        sequencer.lock_acquired = false;
    }

    }else {
        println!("Let's please wait for some more transactions...")
    }
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

fn process_transaction_on_svm(
    client: &RpcClient,
    transaction: &Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    // Send and confirm the transaction
    let signature = client.send_and_confirm_transaction(transaction)?;

    Ok(signature.to_string())
}

fn send_transaction_to_solana(
    client: &RpcClient,
    transaction: &Transaction,
) -> Result<String, Box<dyn std::error::Error>> {
    // Send and confirm the transaction
    let signature = client.send_and_confirm_transaction(transaction)?;

    Ok(signature.to_string())
}

