use anyhow::{Context, Result};
use indexmap::IndexMap;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

const CLOCKWORK_CRATE_PREFIX: &str = "mat-clockwork";

pub fn order_crates_for_publishing() -> Result<Vec<(String, PathBuf)>> {
    let metadata = load_metadata()?;
    let packages = metadata["packages"]
        .as_array()
        .context("Failed to parse packages array")?;

    let mut manifest_path = IndexMap::new();
    let mut dependency_graph = IndexMap::new();
    for pkg in packages {
        let pkg_name = pkg["name"]
            .as_str()
            .context("Failed to parse package name")?
            .to_string();
        let pkg_version = pkg["version"]
            .as_str()
            .context("Failed to parse package version")?
            .to_string();
        println!("{} {}", pkg_name, pkg_version);
        let pkg_manifest_path = pkg["manifest_path"]
            .as_str()
            .context("Failed to parse package manifest path")?
            .to_string();
        let pkg_dependencies = pkg["dependencies"]
            .as_array()
            .context("Failed to parse package dependencies")?;

        // Check if the crate is marked as unpublishable and skip to the next crate if so
        if pkg["publish"].as_array().is_some() {
            println!("{} is marked as unpublishable", pkg_name);
            continue;
        }

        manifest_path.insert(pkg_name.clone(), PathBuf::from(pkg_manifest_path));

        let solana_dependencies: Vec<String> = pkg_dependencies
            .iter()
            .map(|x| {
                x["name"]
                    .as_str()
                    .context("Failed to parse dependency name")
                    .map(|s| s.to_string())
            })
            .collect::<Result<Vec<String>>>()?
            .into_iter()
            .filter(|x| x.starts_with(CLOCKWORK_CRATE_PREFIX))
            .collect();

        dependency_graph.insert(pkg_name, solana_dependencies);
    }

    let mut sorted_dependency_graph = Vec::new();
    while !dependency_graph.is_empty() {
        let mut deleted_packages = HashSet::new();
        for (package, dependencies) in &dependency_graph {
            if dependencies.iter().all(|dep| !dependency_graph.contains_key(dep)) {
                sorted_dependency_graph.push((package.clone(), manifest_path[package].clone()));
                deleted_packages.insert(package.clone());
            }
        }

        if deleted_packages.is_empty() {
            anyhow::bail!(
                "Error: Circular dependency suspected between these packages:\n{}",
                dependency_graph
                    .keys()
                    .map(|pkg| format!("{}\n", pkg))
                    .collect::<String>()
            );
        }

        dependency_graph.retain(|package, _| !deleted_packages.contains(package));
    }

    Ok(sorted_dependency_graph)
}

fn load_metadata() -> Result<serde_json::Value> {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version=1")
        .output()
        .context("Failed to execute cargo metadata")?;
    let stdout = String::from_utf8(output.stdout).context("Failed to convert metadata to string")?;
    let metadata = serde_json::from_str(&stdout).context("Failed to parse metadata JSON")?;
    Ok(metadata)
}
