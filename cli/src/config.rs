use {
    serde::Serialize,
    solana_sdk::commitment_config::CommitmentConfig,
    std::{
        env,
        path::PathBuf,
        time::Duration,
    },
};

pub const DEFAULT_RPC_TIMEOUT_SECONDS: Duration = Duration::from_secs(30);
pub const DEFAULT_CONFIRM_TX_TIMEOUT_SECONDS: Duration = Duration::from_secs(5);
// pub const RELEASE_BASE_URL: &str = "http://localhost:8000/";
pub const RELAYER_URL: &str = "http://localhost:8000/";

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

    pub fn default_home() -> Option<PathBuf> {
        dirs_next::home_dir().map(|mut path| {
            path.extend([".config", "clockwork"]);
            path
        })
    }

    pub fn default_runtime_dir() -> Option<PathBuf> {
        Self::default_home().map(|mut path| {
            path.extend(["localnet", "runtime_deps"]);
            path
        })
    }

    pub fn runtime_path(filename: &str) -> String {
        Self::default_runtime_dir()
            .map(|mut path| {
                path.push(filename);
                path
            })
            .expect(&format!("Unable to find location of {}", filename))
            .into_os_string()
            .into_string()
            .unwrap()
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
            .into_os_string()
            .into_string()
            .unwrap()
    }

    pub fn geyser_config_path() -> String {
        Self::runtime_path("geyser-plugin-config.json")
    }

    pub fn geyser_lib_path() -> String {
        Self::runtime_path("libclockwork_plugin.dylib")
    }
}

impl CliConfig {
    // #[tokio::main]
    // fn detect_target_triplet() -> String {
    //     //TODO: FIXME
    //     return "x86_64-unknown-linux-gnu".to_string();
    // }

    // pub fn localnet_release_archive_url() -> String {
    //     let filename = Self::archive_filename();
    //     format!("{}/{}", RELEASE_BASE_URL, &filename)
    // }
    //
    // pub fn archive_filename() -> String {
    //     let target_triplet = Self::detect_target_triplet();
    //     format!("clockwork-geyser-plugin-release-{}.tar.bz2", target_triplet)
    // }
}

// TODO: use from plugin crate instead
static DEFAULT_TRANSACTION_TIMEOUT_THRESHOLD: u64 = 150;
static DEFAULT_THREAD_COUNT: usize = 10;

#[derive(Clone, Debug, Serialize)]
pub struct GeyserPluginConfig {
    pub keypath: String,
    pub libpath: String,
    pub thread_count: usize,
    pub transaction_timeout_threshold: u64,
    pub worker_id: u64,
    pub sentry_url: Option<String>,
}

impl Default for GeyserPluginConfig {
    fn default() -> Self {
        Self {
            libpath: CliConfig::geyser_lib_path(),
            keypath: CliConfig::signatory_path(),
            sentry_url: None,
            transaction_timeout_threshold: DEFAULT_TRANSACTION_TIMEOUT_THRESHOLD,
            thread_count: DEFAULT_THREAD_COUNT,
            worker_id: 0,
        }
    }
}

//
// fn get_clockwork_config() -> Result<serde_yaml::Value> {
//     let clockwork_config_path = dirs_next::home_dir()
//         .map(|mut path| {
//             path.extend(&[".config", "solana", "clockwork", "config.yml"]);
//             path.to_str().unwrap().to_string()
//         })
//         .unwrap();
//     let f = std::fs::File::open(clockwork_config_path)?;
//     let clockwork_config: serde_yaml::Value = serde_yaml::from_reader(f)?;
//     Ok(clockwork_config)
// }
