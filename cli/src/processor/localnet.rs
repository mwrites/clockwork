#[allow(deprecated)]
use {
    crate::{
        config::CliConfig,
        errors::CliError,
        parser::ProgramInfo,
    },
    anyhow::{
        Context,
        Result,
    },
    clockwork_client::{
        network::state::ConfigSettings,
        thread::state::{
            Thread,
            Trigger,
        },
        Client,
    },
    solana_sdk::{
        native_token::LAMPORTS_PER_SOL,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{
            read_keypair_file,
            Keypair,
            Signer,
        },
        system_instruction,
    },
    spl_associated_token_account::{
        create_associated_token_account,
        get_associated_token_address,
    },
    spl_token::{
        instruction::{
            initialize_mint,
            mint_to,
        },
        state::Mint,
    },
    std::fs,
    std::process::{
        Child,
        Command,
    },
};

pub fn start(
    client: &Client,
    clone_addresses: Vec<Pubkey>,
    network_url: Option<String>,
    program_infos: Vec<ProgramInfo>,
) -> Result<(), CliError> {
    // Create Geyser Plugin Config file
    create_geyser_plugin_config().map_err(|err| CliError::FailedLocalnet(err.to_string()))?;

    // Start the validator
    let validator_process =
        &mut start_test_validator(client, program_infos, network_url, clone_addresses)
            .map_err(|err| CliError::FailedLocalnet(err.to_string()))?;

    // Initialize Clockwork
    let mint_pubkey =
        mint_clockwork_token(client).map_err(|err| CliError::FailedTransaction(err.to_string()))?;
    super::initialize::initialize(client, mint_pubkey)
        .map_err(|err| CliError::FailedTransaction(err.to_string()))?;
    register_worker(client).map_err(|err| CliError::FailedTransaction(err.to_string()))?;
    create_threads(client, mint_pubkey)
        .map_err(|err| CliError::FailedTransaction(err.to_string()))?;

    // Wait for process to be killed.
    _ = validator_process.wait();

    Ok(())
}

fn mint_clockwork_token(client: &Client) -> Result<Pubkey> {
    // Calculate rent and pubkeys
    let mint_keypair = Keypair::new();
    let mint_rent = client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .unwrap_or(0);
    let token_account_pubkey =
        get_associated_token_address(&client.payer_pubkey(), &mint_keypair.pubkey());

    // Build ixs
    let ixs = vec![
        // Create mint account
        system_instruction::create_account(
            &client.payer_pubkey(),
            &mint_keypair.pubkey(),
            mint_rent,
            Mint::LEN as u64,
            &spl_token::ID,
        ),
        initialize_mint(
            &spl_token::ID,
            &mint_keypair.pubkey(),
            &client.payer_pubkey(),
            None,
            8,
        )
        .unwrap(),
        // Create associated token account
        #[allow(deprecated)]
        create_associated_token_account(
            &client.payer_pubkey(),
            &client.payer_pubkey(),
            &mint_keypair.pubkey(),
        ),
        // Mint 10 tokens to the local user
        mint_to(
            &spl_token::ID,
            &mint_keypair.pubkey(),
            &token_account_pubkey,
            &client.payer_pubkey(),
            &[&client.payer_pubkey()],
            1000000000,
        )
        .unwrap(),
    ];

    // Submit tx
    client
        .send_and_confirm(&ixs, &[client.payer(), &mint_keypair])
        .context("mint_clockwork_token failed")?;

    Ok(mint_keypair.pubkey())
}

fn register_worker(client: &Client) -> Result<()> {
    // Create the worker
    let signatory = read_keypair_file(CliConfig::signatory_path()).map_err(|err| {
        CliError::FailedLocalnet(format!(
            "Unable to read keypair {}: {}",
            CliConfig::signatory_path(),
            err
        ))
    })?;

    client
        .airdrop(&signatory.pubkey(), LAMPORTS_PER_SOL)
        .context("airdrop to signatory failed")?;
    super::worker::create(client, signatory, true).context("worker::create failed")?;

    // Delegate stake to the worker
    super::delegation::create(client, 0).context("delegation::create failed")?;
    super::delegation::deposit(client, 100000000, 0, 0).context("delegation::deposit failed")?;
    Ok(())
}

fn create_threads(client: &Client, mint_pubkey: Pubkey) -> Result<()> {
    // Create epoch thread.
    let epoch_thread_id = "clockwork.network.epoch";
    let epoch_thread_pubkey = Thread::pubkey(client.payer_pubkey(), epoch_thread_id.into());
    let ix_a = clockwork_client::thread::instruction::thread_create(
        LAMPORTS_PER_SOL,
        client.payer_pubkey(),
        epoch_thread_id.into(),
        vec![
            clockwork_client::network::job::distribute_fees(epoch_thread_pubkey).into(),
            clockwork_client::network::job::process_unstakes(epoch_thread_pubkey).into(),
            clockwork_client::network::job::stake_delegations(epoch_thread_pubkey).into(),
            clockwork_client::network::job::take_snapshot(epoch_thread_pubkey).into(),
            clockwork_client::network::job::increment_epoch(epoch_thread_pubkey).into(),
            clockwork_client::network::job::delete_snapshot(epoch_thread_pubkey).into(),
        ],
        client.payer_pubkey(),
        epoch_thread_pubkey,
        Trigger::Cron {
            schedule: "0 * * * * * *".into(),
            skippable: true,
        },
    );

    // Create hasher thread.
    let hasher_thread_id = "clockwork.network.hasher";
    let hasher_thread_pubkey = Thread::pubkey(client.payer_pubkey(), hasher_thread_id.into());
    let ix_b = clockwork_client::thread::instruction::thread_create(
        LAMPORTS_PER_SOL,
        client.payer_pubkey(),
        hasher_thread_id.into(),
        vec![
            clockwork_client::network::instruction::registry_nonce_hash(hasher_thread_pubkey)
                .into(),
        ],
        client.payer_pubkey(),
        hasher_thread_pubkey,
        Trigger::Cron {
            schedule: "*/15 * * * * * *".into(),
            skippable: true,
        },
    );

    // Update config with thread pubkeys
    let ix_c = clockwork_client::network::instruction::config_update(
        client.payer_pubkey(),
        ConfigSettings {
            admin: client.payer_pubkey(),
            epoch_thread: epoch_thread_pubkey,
            hasher_thread: hasher_thread_pubkey,
            mint: mint_pubkey,
        },
    );

    client
        .send_and_confirm(&vec![ix_a], &[client.payer()])
        .context(format!(
            "Failed to create thread: {} or update config",
            epoch_thread_id,
        ))?;
    client
        .send_and_confirm(&vec![ix_b, ix_c], &[client.payer()])
        .context(format!("Failed to create thread: {}", hasher_thread_id))?;

    Ok(())
}

fn create_geyser_plugin_config() -> Result<()> {
    let config = clockwork_plugin::geyser_config::PluginConfig {
        keypath: Some(CliConfig::signatory_path()),
        libpath: Some(CliConfig::geyser_lib_path()),
        ..Default::default()
    };

    let content = serde_json::to_string_pretty(&config)
        .context("Unable to serialize PluginConfig to json")?;
    let path = CliConfig::geyser_config_path();
    fs::write(&path, content).context(format!("Unable to serialize PluginConfig to {}", path))?;
    Ok(())
}

fn start_test_validator(
    client: &Client,
    program_infos: Vec<ProgramInfo>,
    network_url: Option<String>,
    clone_addresses: Vec<Pubkey>,
) -> Result<Child> {
    println!("Starting test validator");

    let path = CliConfig::runtime_path("solana-test-validator".clone()).clone();
    let cmd = &mut Command::new(path);
    cmd.arg("-r")
        .bpf_program(clockwork_client::network::ID, "network")
        .bpf_program(clockwork_client::thread::ID, "thread")
        .bpf_program(clockwork_client::webhook::ID, "webhook")
        .network_url(network_url)
        .clone_addresses(clone_addresses)
        .add_programs_with_path(program_infos)
        .geyser_plugin_config();

    let mut process = cmd
        .spawn()
        .context(format!("start validator command: {:#?}", cmd))?;

    // Wait for the validator to become healthy
    let ms_wait = 10_000;
    let mut count = 0;
    while count < ms_wait {
        match client.get_block_height() {
            Err(_err) => {
                std::thread::sleep(std::time::Duration::from_millis(1));
                count += 1;
            }
            Ok(slot) => {
                if slot > 0 {
                    println!("Got a slot: {}", slot);
                    break;
                }
            }
        }
    }
    if count == ms_wait {
        process.kill()?;
        std::process::exit(1);
    }

    // Wait 1 extra second for safety before submitting txs
    std::thread::sleep(std::time::Duration::from_secs(1));

    Ok(process)
}

trait TestValidatorHelpers {
    fn add_programs_with_path(&mut self, program_infos: Vec<ProgramInfo>) -> &mut Command;
    fn bpf_program(&mut self, program_id: Pubkey, program_name: &str) -> &mut Command;
    fn geyser_plugin_config(&mut self) -> &mut Command;
    fn network_url(&mut self, url: Option<String>) -> &mut Command;
    fn clone_addresses(&mut self, clone_addresses: Vec<Pubkey>) -> &mut Command;
}

impl TestValidatorHelpers for Command {
    fn add_programs_with_path(&mut self, program_infos: Vec<ProgramInfo>) -> &mut Command {
        for program_info in program_infos {
            self.arg("--bpf-program")
                .arg(program_info.program_id.to_string())
                .arg(program_info.program_path);
        }

        self
    }
    fn bpf_program(&mut self, program_id: Pubkey, program_name: &str) -> &mut Command {
        let filename = format!("clockwork_{}_program.so", program_name);
        self.arg("--bpf-program")
            .arg(program_id.to_string())
            .arg(CliConfig::runtime_path(filename.as_str()))
    }

    fn geyser_plugin_config(&mut self) -> &mut Command {
        self.arg("--geyser-plugin-config")
            .arg(CliConfig::geyser_config_path())
    }

    fn network_url(&mut self, url: Option<String>) -> &mut Command {
        if let Some(url) = url {
            self.arg("--url").arg(url);
        }
        self
    }

    fn clone_addresses(&mut self, clone_addresses: Vec<Pubkey>) -> &mut Command {
        for clone_address in clone_addresses {
            self.arg("--clone").arg(clone_address.to_string());
        }
        self
    }
}
