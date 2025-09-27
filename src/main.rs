use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};
use std::str::FromStr;

fn main() {
    println!("Connecting to Solana Devnet...");

    let rpc_url = String::from("https://api.devnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    // println!("Connected to  Devnet...");

    // match client.get_slot() {
    //     Ok(slot) => println!("Current slot is: {}", slot),
    //     Err(e) => println!("Error getting slot: {}", e),
    // }

    let address_sol = "So11111111111111111111111111111111111111112";

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
}
