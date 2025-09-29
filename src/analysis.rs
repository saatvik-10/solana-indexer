use crate::models::TransactionAnalysis;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcTransactionConfig};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::Signature;
use solana_transaction_status::UiTransactionEncoding;
use std::str::FromStr;

pub fn analyze_transaction(
    client: &RpcClient,
    sig_info: &solana_client::rpc_response::RpcConfirmedTransactionStatusWithSignature,
) -> TransactionAnalysis {
    let mut analysis = TransactionAnalysis {
        fee: 0,
        status: if sig_info.err.is_none() {
            "Success".to_string()
        } else {
            "Failed".to_string()
        },
        value_moved: 0,
    };

    println!("Signature: {}", sig_info.signature);
    println!("Slot: {}", sig_info.slot);
    println!("Status: {}", analysis.status);

    match Signature::from_str(&sig_info.signature) {
        Ok(sig) => {
            let config = RpcTransactionConfig {
                encoding: Some(UiTransactionEncoding::Json),
                commitment: Some(CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            };

            if let Ok(transaction_response) = client.get_transaction_with_config(&sig, config) {
                if let Some(meta) = transaction_response.transaction.meta {
                    analysis.fee = meta.fee;
                    println!("Fee: {} lamports", analysis.fee);

                    if meta.pre_balances.len() == meta.post_balances.len() {
                        let mut total_change = 0i64;
                        for (pre, post) in meta.pre_balances.iter().zip(meta.post_balances.iter()) {
                            let change = *post as i64 - *pre as i64;
                            total_change += change.abs();
                        }
                        analysis.value_moved = total_change / 2;
                        println!(
                            "Total Value Moved: {} lamports ({:.6} SOL)",
                            analysis.value_moved,
                            analysis.value_moved as f64 / 1_000_000_000.0
                        );
                    }
                }
            }
        }
        Err(e) => println!("Could not parse signature: {}", e),
    }
    analysis
}
