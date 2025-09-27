use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;

fn main() {
    println!("Connecting to Solana Devnet...");

    let rpc_url = String::from("https://api.devnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

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
}
