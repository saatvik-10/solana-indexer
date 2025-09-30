pub mod analysis;
mod db;
pub mod models;

use crate::analysis::analyze_transaction;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;
use std::{str::FromStr, thread, time::Duration};

fn main() -> db::SqlResult<()> {
    println!("Connecting to Solana Devnet...");

    let rpc_url = String::from("https://api.devnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    let db_conn = db::init_db()?;

    println!("\nBasic devnet connection...\n");
    println!("Connected to  Devnet...");

    match client.get_slot() {
        Ok(slot) => println!("Current slot is: {}", slot),
        Err(e) => println!("Error getting slot: {}", e),
    }

    let address_sol = "So11111111111111111111111111111111111111112";

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
    let mut last_seen_sig = String::new();

    match Pubkey::from_str(address_sol) {
        Ok(pubkey) => {
            println!("Address {} being monitored", address_sol);
            println!("Polling transaction every 10 seconds...");

            loop {
                match client.get_signatures_for_address(&pubkey) {
                    Ok(sigs) => {
                        if !sigs.is_empty() {
                            let latest_sig = &sigs[0];

                            if latest_sig.signature != last_seen_sig {
                                println!("New transaction detected!");
                                let analyze_res = analyze_transaction(&client, latest_sig);

                                db::save_txn(
                                    &db_conn,
                                    &latest_sig.signature,
                                    latest_sig.slot,
                                    latest_sig.block_time.unwrap_or(0),
                                    analyze_res.fee,
                                    &analyze_res.status,
                                    analyze_res.value_moved,
                                )?;

                                last_seen_sig = latest_sig.signature.clone();
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
                }
                thread::sleep(Duration::from_secs(10));
            }
        }
        Err(e) => println!("Address is invalid: {}", e),
    }
    Ok(())
}
