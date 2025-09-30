pub mod analysis;
mod db;
pub mod models;

use crate::analysis::analyze_transaction;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::{collections::HashMap, str::FromStr, thread, time::Duration};

fn main() -> db::SqlResult<()> {
    println!("Connecting to Solana Devnet...");

    let rpc_url = String::from("https://api.mainnet-beta.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let db_conn = db::init_db()?;

    println!("\nBasic devnet connection...\n");
    println!("Connected to  Devnet...");

    match client.get_slot() {
        Ok(slot) => println!("Current slot is: {}", slot),
        Err(e) => println!("Error getting slot: {}", e),
    }

    let address_sol = "So11111111111111111111111111111111111111112";

    let addresses: Vec<String> = vec![
        "So11111111111111111111111111111111111111112".to_string(),
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
        "DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263".to_string(),
        "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8".to_string(),
        "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTp1".to_string(),
    ];

    let mut last_seen: HashMap<String, String> = HashMap::new();

    for address in &addresses {
        let _ = last_seen.insert(address.clone(), String::new());
    }

    println!("\nQuerying Transaction...\n");
    match Pubkey::from_str(address_sol) {
        Ok(pubkey) => {
            println!("Checking address: {}", address_sol);

            match client.get_balance(&pubkey) {
                Ok(balance) => {
                    let balance_sol = balance as f64 / 1000_000_000.0;
                    println!("Balance: {} lamports ({:.9} SOL)", balance, balance_sol)
                }
                Err(e) => println!("Error fetching balance: {}", e),
            }
        }
        Err(e) => println!("Invalid Address: {}", e),
    }

    println!("\nRecent Trasaction Sigs...\n");
    match Pubkey::from_str(address_sol) {
        Ok(pubkey) => {
            println!("Fetching transaction history for: {}", address_sol);

            match client.get_signatures_for_address(&pubkey) {
                Ok(sigs) => {
                    let limited_sigs = sigs.iter().take(5);

                    println!("Found {} recent transactions:\n", sigs.len());

                    for (i, sig_info) in limited_sigs.enumerate() {
                        println!("Transaction #{}", i + 1);
                        println!("Signature: {}", sig_info.signature);
                        println!("Slot: {}", sig_info.slot);
                        println!("Block Time: {:?}", sig_info.block_time);

                        if let Some(err) = &sig_info.err {
                            println!("Error: {:?}", err);
                        } else {
                            println!("Success");
                        }
                        println!(".......................");
                    }
                }
                Err(e) => println!("Error fetching signatures {}", e),
            }
        }
        Err(e) => println!("Invalid wallet address: {}", e),
    }

    println!("\nFetching advance transaction history...\n");
    match Pubkey::from_str(address_sol) {
        Ok(pubkey) => {
            println!("Fetching transaction details for: {}", address_sol);

            //recent transaction sigs
            match client.get_signatures_for_address(&pubkey) {
                Ok(sigs) => {
                    if sigs.is_empty() {
                        println!("No transaction found!");
                        return Ok(());
                    }

                    //details for recent trasaction
                    let recent_sig = &sigs[0];
                    println!("Analyzing the most recent transaction...");
                    println!("Signature: {}", recent_sig.signature);

                    //parsing the signature string
                    match Signature::from_str(&recent_sig.signature) {
                        Ok(sig) => {
                            let config = RpcTransactionConfig {
                                encoding: Some(UiTransactionEncoding::Json),
                                commitment: Some(CommitmentConfig::confirmed()),
                                max_supported_transaction_version: Some(0),
                            };

                            //fetching detailed transaction data
                            match client.get_transaction_with_config(&sig, config) {
                                Ok(transaction_response) => {
                                    println!("\nTransaction Details:");

                                    println!("Slots: {}", transaction_response.slot);
                                    println!("Block Time: {:?}", transaction_response.block_time);

                                    //metadata
                                    if let Some(meta) = transaction_response.transaction.meta {
                                        println!("Fee: {} lamports", meta.fee);
                                        println!(
                                            "Status: {}",
                                            if meta.err.is_none() {
                                                "Success"
                                            } else {
                                                "Failed"
                                            }
                                        );

                                        //balance changes
                                        if meta.pre_balances.len() == meta.post_balances.len() {
                                            println!("Balance Changes:");

                                            for (i, (pre, post)) in meta
                                                .pre_balances
                                                .iter()
                                                .zip(meta.post_balances.iter())
                                                .enumerate()
                                            {
                                                let change = *post as i64 - *pre as i64;

                                                if change != 0 {
                                                    println!("Account: {}: {} lamports", i, change);
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => println!("Error fetching transaction details: {}", e),
                            }
                        }
                        Err(e) => println!("Error parsing signature: {}", e),
                    }
                }
                Err(e) => println!("Error fetching signatures: {}", e),
            }
        }
        Err(e) => println!("Invalid Address: {}", e),
    }

    println!("\nContinuous Transaction Monitor...\n");

    println!(
        "Addresses {} being monitored: {:?}",
        addresses.len(),
        addresses
    );
    println!("Polling transaction every 10 seconds...");

    loop {
        for address in &addresses {
            match Pubkey::from_str(address) {
                Ok(pubkey) => match client.get_signatures_for_address(&pubkey) {
                    Ok(sigs) => {
                        if !sigs.is_empty() {
                            let latest_sig = &sigs[0];
                            let last = last_seen.get(address).unwrap();

                            if latest_sig.signature != *last {
                                println!("New transaction detected!");
                                let analyze_res = analyze_transaction(&client, latest_sig);

                                db::save_txn(
                                    &db_conn,
                                    address,
                                    &latest_sig.signature,
                                    latest_sig.slot,
                                    latest_sig.block_time.unwrap_or(0),
                                    analyze_res.fee,
                                    &analyze_res.status,
                                    analyze_res.value_moved,
                                )?;

                                last_seen.insert(address.clone(), latest_sig.signature.clone());
                                println!("..........................................");
                            } else {
                                println!("No new transaction found...");
                                println!(
                                    "Last polled at: {}",
                                    chrono::Utc::now().format("%H:%M:%S")
                                );
                                println!("..........................................");
                            }
                        }
                    }
                    Err(e) => println!("Error fetching signatures: {}", e),
                },
                Err(e) => println!("Address is invalid: {}", e),
            }
        }
        thread::sleep(Duration::from_secs(10));
    }
}
