mod crate_registry;
mod order_crates;

use order_crates::order_crates_for_publishing;
use crate_registry::{is_crate_version_uploaded, publish_crate};

use anyhow::{bail, Context, Result};
use clap::{Arg, App};
use std::{
    env,
    thread::sleep,
    time::Duration,
};
use std::collections::HashSet;
use crate::order_crates::CrateInfo;

pub const CHECK_CRATES_IO_RETRIES: u32 = 30;
pub const ENV_VAR_CRATES_IO_TOKEN: &str = "CRATES_IO_TOKEN";
pub const ENV_VAR_CI_TAG: &str = "CI_TAG";

fn main() -> Result<()> {
    let matches = App::new("Workspace Publisher")
        .version("1.0")
        .author("mwrites <mwrites.pub@pm.me>")
        .arg(
            Arg::new("workspace-crate-prefix")
                .short('p')
                .long("crate-prefix")
                .takes_value(true)
                .required(true)
                .help("The prefix of the crates to publish, e.g. 'my-repo-crate-'")
        )
        .arg(
            Arg::new("dry-run")
                .short('d')
                .long("dry-run")
                .help("Run without uploading")
        )
        .arg(
            Arg::new("show-order")
                .short('o')
                .long("show-order")
                .help("Only display the order of crates to be published")
        )
        .arg(
            Arg::new("token")
                .short('t')
                .long("token")
                .help("Specify the token to use instead of CRATES_IO_TOKEN environment variable")
                .takes_value(true),
        )
        .arg(
            Arg::new("version")
                .short('v')
                .long("version")
                .help("Specify the version to use instead of CI_TAG environment variable")
                .takes_value(true),
        )
        .arg(
            Arg::new("ignore-crate")
                .short('i')
                .long("ignore-crate")
                .help("Ignore a crate from being published (arg can be supplied multiple times)")
                .takes_value(true)
                .multiple(true),
        )
        .get_matches();

    let crate_prefix = matches.value_of("workspace-crate-prefix").unwrap();
    let dry_run = matches.is_present("dry-run");
    let show_order = matches.is_present("show-order");

    let ignored_crates: HashSet<String> = matches
        .values_of("ignore-crate")
        .map(|values| values.map(|s| s.to_string()).collect())
        .unwrap_or_default();

    // Get the crates to publish in the correct order and filter out any ignored crates
    let crates = order_crates_for_publishing(ignored_crates, crate_prefix)?;
    if show_order {
        println!("Will publish the crates in the following order:");
        for (i, _crate) in crates.iter().enumerate() {
            println!("{}. {}", i, _crate.name);
        }
        return Ok(());
    }

    // Get CI_TAG and CRATES_IO_TOKEN environment variables or command line arguments
    let version = matches
        .value_of("version")
        .map(|v| v.to_string())
        .unwrap_or_else(|| env::var(ENV_VAR_CI_TAG).expect("Failed to get CI_TAG environment variable"))
        .replacen('v', "", 1);
    let crates_io_token = matches
        .value_of("token")
        .map(|t| t.to_string())
        .unwrap_or_else(|| env::var(ENV_VAR_CRATES_IO_TOKEN).expect("Failed to get CRATES_IO_TOKEN environment variable"));


    publish_workspace(dry_run, &crates_io_token, crates, version)?;
    Ok(())
}

fn publish_workspace(dry_run: bool, crates_io_token: &str, crates: Vec<CrateInfo>, version: String) -> Result<()> {
    // Iterate through each Cargo.toml file
    for _crate in &crates {
        println!("Processing: {} --- {:?}", _crate.name, _crate.manifest_path);

        // Check if the version in the Cargo.toml matches the expected crate version
        if _crate.version != version {
            bail!("Error: {:?} version is not {}", _crate.name, version);
        }

        // Check if the crate version is already on crates.io
        if is_crate_version_uploaded(&_crate.name, &_crate.version) {
            println!("{} version {} is already on crates.io", _crate.name, _crate.version);
            continue;
        }

        // Publish the crate
        publish_crate(dry_run, &crates_io_token, &_crate.manifest_path)
            .context(format!("Failed to publish crate {}", _crate.name))?;

        // Retry checking if the crate version is uploaded to crates.io and available for download
        println!("Waiting for crate '{}' to appear in crates.io", _crate.name);
        let num_retries = CHECK_CRATES_IO_RETRIES;
        for i in 1..=num_retries {
            println!("...Attempt {} of {}", i, num_retries);
            if is_crate_version_uploaded(&_crate.name, &_crate.version) {
                println!(
                    "-> Found {} version {} on crates.io REST API",
                    _crate.name, _crate.version
                );
                break;
            } else {
                println!(
                    "...Did not find {} version {} on crates.io. Sleeping for 2 seconds.",
                    _crate.name, _crate.version
                );
                sleep(Duration::from_secs(2));
            }
        }
    }
    println!("All done!");
    Ok(())
}
