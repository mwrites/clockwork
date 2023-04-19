fn main() {
    let metadata = cargo_metadata::MetadataCommand::new().exec().unwrap();
    let geyser_interface_version = metadata
        .packages
        .iter()
        .find(|p| p.name == "solana-geyser-plugin-interface")
        .expect("Unable to parse solana-geyser-plugin-interface version using cargo metadata")
        .version
        .to_string();
    println!(
        "cargo:rustc-env=GEYSER_INTERFACE_VERSION={}",
        geyser_interface_version
    );
}
