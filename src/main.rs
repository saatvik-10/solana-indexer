use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;

fn main() {
    println!("Connecting to Solana Devnet...");

    let rpc_url = String::from("https://api.devnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());

    println!("Connected to  Devnet...");

    match client.get_slot() {
        Ok(slot) => println!("Current slot is: {}", slot),
        Err(e) => println!("Error getting slot: {}", e),
    }
}
