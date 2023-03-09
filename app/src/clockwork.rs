use anchor_lang::{prelude::Pubkey, Discriminator};
use clockwork_sdk::state::Thread;
use dotenv_codegen::dotenv;
use solana_client_wasm::{
    solana_sdk::{account::Account, commitment_config::CommitmentConfig},
    utils::{
        rpc_config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
        rpc_filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
    },
    WasmClient,
};
use solana_extra_wasm::account_decoder::UiAccountEncoding;

pub async fn get_threads() -> Vec<(Thread, Account)> {
    const HELIUS_API_KEY: &str = dotenv!("HELIUS_API_KEY");
    let url = format!("https://rpc.helius.xyz/?api-key={}", HELIUS_API_KEY);
    let helius_rpc_endpoint = url.as_str();
    let client = WasmClient::new(helius_rpc_endpoint);
    // let client = WasmClient::new("http://74.118.139.244:8899");

    let accounts = client
        .get_program_accounts_with_config(
            &clockwork_sdk::ID,
            RpcProgramAccountsConfig {
                filters: Some(vec![RpcFilterType::Memcmp(Memcmp {
                    offset: 0,
                    bytes: MemcmpEncodedBytes::Bytes(Thread::discriminator().to_vec()),
                    encoding: None,
                })]),
                account_config: RpcAccountInfoConfig {
                    encoding: Some(UiAccountEncoding::Base64),
                    data_slice: None,
                    commitment: Some(CommitmentConfig::finalized()),
                    min_context_slot: None,
                },
                with_context: None,
            },
        )
        .await
        .unwrap()
        .iter()
        .map(|acc| (Thread::try_from(acc.1.data.clone()).unwrap(), acc.1.clone()))
        .collect::<Vec<(Thread, Account)>>();
    accounts[0..10].to_vec()
}

pub async fn get_thread(pubkey: Pubkey) -> Thread {
    // let client = WasmClient::new("http://74.118.139.8899");
    const HELIUS_API_KEY: &str = dotenv!("HELIUS_API_KEY");
    let url = format!("https://rpc.helius.xyz/?api-key={}", HELIUS_API_KEY);
    let helius_rpc_endpoint = url.as_str();
    let client = WasmClient::new(helius_rpc_endpoint);

    let account = client
        .get_account_with_config(
            &pubkey,
            RpcAccountInfoConfig {
                encoding: Some(UiAccountEncoding::Base64),
                data_slice: None,
                commitment: Some(CommitmentConfig::finalized()),
                min_context_slot: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    Thread::try_from(account.data).unwrap()
}