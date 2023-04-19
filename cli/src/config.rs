use {
    solana_sdk::commitment_config::CommitmentConfig,
    std::{
        env,
        path::PathBuf,
        time::Duration,
    },
};

pub const DEFAULT_RPC_TIMEOUT_SECONDS: Duration = Duration::from_secs(30);
pub const DEFAULT_CONFIRM_TX_TIMEOUT_SECONDS: Duration = Duration::from_secs(5);
pub const RELAYER_URL: &str = "http://localhost:8000/";
pub const CLOCKWORK_RELEASE_BASE_URL: &str =
    "http://localhost:8000/clockwork-xyz/clockwork/releases/download";
// pub const CLOCKWORK_RELEASE_BASE_URL = "https://github.com/clockwork-xyz/clockwork/releases/download"
pub const CLOCKWORK_ARCHIVE_PREFIX: &str = "clockwork-geyser-plugin-release/lib";
pub const SOLANA_RELEASE_BASE_URL: &str =
    "http://localhost:8000/solana-labs/solana/releases/download";
// pub const SOLANA_RELEASE_BASE_URL = "https://github.com/solana-labs/solana/releases/download";
pub const SOLANA_ARCHIVE_PREFIX: &str = "solana-release/bin";

/// The combination of solana config file and our own config file
#[derive(Debug)]
pub struct CliConfig {
    pub json_rpc_url: String,
    pub websocket_url: String,
    pub relayer_url: String,
    pub keypair_path: String,
    pub rpc_timeout: Duration,
    pub commitment: CommitmentConfig,
    pub confirm_transaction_initial_timeout: Duration,
}

impl CliConfig {
    pub fn load() -> Self {
        let solana_config_file = solana_cli_config::CONFIG_FILE.as_ref().unwrap().as_str();
        let solana_config = solana_cli_config::Config::load(solana_config_file).unwrap();
        CliConfig {
            json_rpc_url: solana_config.json_rpc_url,
            websocket_url: solana_config.websocket_url,
            relayer_url: RELAYER_URL.to_owned(),
            keypair_path: solana_config.keypair_path,
            rpc_timeout: DEFAULT_RPC_TIMEOUT_SECONDS,
            commitment: CommitmentConfig::confirmed(),
            confirm_transaction_initial_timeout: DEFAULT_CONFIRM_TX_TIMEOUT_SECONDS,
        }
    }

    pub fn default_home() -> PathBuf {
        dirs_next::home_dir()
            .map(|mut path| {
                path.extend([".config", "clockwork"]);
                path
            })
            .unwrap()
    }

    pub fn default_runtime_dir() -> PathBuf {
        let mut path = Self::default_home();
        path.extend(["localnet", "runtime_deps"]);
        path
    }

    pub fn runtime_path(filename: &str) -> String {
        Self::default_runtime_dir().join(filename).to_string()
    }

    /// This assumes the path for the signatory keypair created by solana-test-validator
    /// is test-ledger/validator-keypair.json
    pub fn signatory_path() -> String {
        env::current_dir()
            .map(|mut path| {
                path.extend(["test-ledger", "validator-keypair.json"]);
                path
            })
            .expect(&format!(
                "Unable to find location of validator-keypair.json"
            ))
            .to_string()
    }

    pub fn geyser_config_path() -> String {
        Self::runtime_path("geyser-plugin-config.json")
    }

    pub fn geyser_lib_path() -> String {
        Self::runtime_path("libclockwork_plugin.dylib")
    }
}

pub trait PathToString {
    fn to_string(&self) -> String;
}

impl PathToString for PathBuf {
    fn to_string(&self) -> String {
        self.clone().into_os_string().into_string().unwrap()
    }
}

// Clockwork Deps Helpers
impl CliConfig {
    // #[tokio::main]
    fn detect_target_triplet() -> String {
        return "aarch64-apple-darwin".to_owned();
        //TODO: FIXME
        // return "x86_64-unknown-linux-gnu".to_string();
    }

    pub fn clockwork_release_url(tag: &str) -> String {
        format!(
            "{}/{}/{}",
            CLOCKWORK_RELEASE_BASE_URL,
            tag,
            &Self::clockwork_release_archive()
        )
    }

    pub fn clockwork_release_archive() -> String {
        let target_triplet = Self::detect_target_triplet();
        format!("clockwork-geyser-plugin-release-{}.tar.bz2", target_triplet)
    }

    pub fn solana_release_url(tag: &str) -> String {
        format!(
            "{}/{}/{}",
            SOLANA_RELEASE_BASE_URL,
            tag,
            &Self::solana_release_archive()
        )
    }

    pub fn solana_release_archive() -> String {
        let target_triplet = Self::detect_target_triplet();
        format!("solana-release-{}.tar.bz2", target_triplet)
    }
}
