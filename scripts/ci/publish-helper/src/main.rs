mod crate_registry;
mod order_crates;

use order_crates::order_crates_for_publishing;
use crate_registry::{is_crate_version_uploaded, publish_crate};

use anyhow::{bail, Context, Result};
use clap::{Arg, App};
use std::{
    env, fs,
    thread::sleep,
    time::Duration,
};

const CHECK_CRATES_IO_RETRIES: u32 = 30;
const ENV_VAR_CRATES_IO_TOKEN: &str = "CRATES_IO_TOKEN";
const ENV_VAR_CI_TAG: &str = "CI_TAG";

fn main() -> Result<()> {
    let matches = App::new("Crate Publisher")
        .version("1.0")
        .author("Your Name <your.email@example.com>")
        .arg(
            Arg::new("dry-run")
                .short('d')
                .long("dry-run")
                // .about("Perform a dry run without actually publishing"),
        )
        .arg(
            Arg::new("show-order")
                .short('o')
                .long("show-order")
        )
        .get_matches();

    let dry_run = matches.is_present("dry-run");
    let show_order = matches.is_present("show-order");

    // Get the ordered list of Cargo.toml
    let crates = order_crates_for_publishing()?;
    if show_order {
        println!("Will publish the crates in the following order:");
        for (i, (crate_name, _)) in crates.iter().enumerate() {
            println!("{}. {}", i, crate_name);
        }
        return Ok(())
    }

    // Set the working directory to the parent of the script's directory
    // let script_dir = Path::new(file!()).parent().unwrap();
    // let root_dir = script_dir.parent().unwrap();
    // env::set_current_dir(&root_dir).map_err(|_| CratePublishError::ChangeWorkingDirectoryError)?;

    // Get CI_TAG and CRATES_IO_TOKEN environment variables
    let ci_tag = env::var(ENV_VAR_CI_TAG)
        .context("Failed to get CI_TAG environment variable")?;
    let crates_io_token = env::var(ENV_VAR_CRATES_IO_TOKEN)
        .context("Failed to get CRATES_IO_TOKEN environment variable")?;

    // Parse CI_TAG into its semantic version components
    let tag_version = semver::Version::parse(&ci_tag)
        .context("Failed to parse CI_TAG into a semantic version")?;

    // Check if CRATES_IO_TOKEN is set
    if crates_io_token.is_empty() {
        bail!("Environment variable '{}' is not defined", ENV_VAR_CRATES_IO_TOKEN);
    }

    // Iterate through each Cargo.toml file
    for (crate_name, cargo_toml_path) in &crates {
        println!("Processing: {} --- {:?}", crate_name, cargo_toml_path);

        // // Read the contents of the Cargo.toml file
        let contents = fs::read_to_string(cargo_toml_path.clone()).expect("Failed to read Cargo.toml");

        // Check if the version in the Cargo.toml matches the expected crate version
        let expected_crate_version = format!("version = \"{}\"", tag_version);
        if !contents.contains(&expected_crate_version) {
            bail!("Error: {:?} version is not {}", crate_name, tag_version);
        }

        // Check if the crate version is already on crates.io
        if is_crate_version_uploaded(crate_name, &tag_version.to_string()) {
            println!("{} version {} is already on crates.io", crate_name, tag_version);
            continue;
        }

        // Publish the crate
        publish_crate(dry_run, &crates_io_token, cargo_toml_path)
            .context(format!("Failed to publish crate {}", crate_name))?;

        // Retry checking if the crate version is uploaded to crates.io and available for download
        println!("Waiting for crate '{}' to appear in crates.io", crate_name);
        let num_retries = CHECK_CRATES_IO_RETRIES;
        for i in 1..=num_retries {
            println!("...Attempt {} of {}", i, num_retries);
            if is_crate_version_uploaded(crate_name, &tag_version.to_string()) {
                println!(
                    "-> Found {} version {} on crates.io REST API",
                    crate_name, tag_version
                );
                break;
            } else {
                println!(
                    "...Did not find {} version {} on crates.io. Sleeping for 2 seconds.",
                    crate_name, tag_version
                );
                sleep(Duration::from_secs(2));
            }
        }
    }
    println!("All done!");
    Ok(())
}
