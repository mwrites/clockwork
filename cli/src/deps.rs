use {
    crate::config::CliConfig,
    anyhow::{
        Context,
        Result,
    },
    bzip2::read::BzDecoder,
    indicatif::{
        ProgressBar,
        ProgressStyle,
    },
    reqwest::blocking::get,
    std::{
        fs::{
            self,
            File,
        },
        io::{self,},
        path::{
            Path,
            PathBuf,
        },
    },
    tar::Archive,
};

pub fn download_and_extract(runtime_dir: &Path) -> Result<()> {
    let filename = CliConfig::archive_filename();
    let dest_path = CliConfig::default_runtime_dir().join(filename);
    download_file(&CliConfig::localnet_release_archive_url(), &dest_path)?;
    extract_archive(&dest_path, runtime_dir)?;
    Ok(())
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    let resp = get(url).context(format!("Failed to download file from {}", url))?;

    let pb = ProgressBar::new(resp.content_length().unwrap_or(0));
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .progress_chars("#>-"));

    let mut source = pb.wrap_read(resp);

    let mut dest = File::create(&dest).context(format!("Failed to create file {:#?}", dest))?;
    io::copy(&mut source, &mut dest)
        .context(format!("Failed to copy data from {} to {:#?}", url, &dest))?;
    pb.finish_with_message("Download complete.");
    Ok(())
}

fn extract_archive(archive_path: &Path, runtime_dir: &Path) -> Result<()> {
    // create runtime dir if necessary
    fs::create_dir_all(runtime_dir)?;

    let file =
        File::open(&archive_path).context(format!("Failed to open file {:#?}", archive_path))?;
    let mut archive = Archive::new(BzDecoder::new(file));
    let prefix = "clockwork-geyser-plugin-release/lib";

    println!("Extracted the following files:");
    archive
        .entries()?
        .filter_map(|e| e.ok())
        .map(|mut entry| -> Result<PathBuf> {
            let path = entry
                .path()?
                .strip_prefix(prefix)
                .context(format!("Failed to strip prefix {}", prefix))?
                .to_owned();
            let target_path = runtime_dir.join(&path);
            entry.unpack(&target_path).context(format!(
                "Failed to unpack {:#?} into {:#?}",
                path, target_path
            ))?;
            Ok(target_path)
        })
        .filter_map(|e| e.ok())
        .for_each(|x| {
            println!("> {}", x.display());
        });
    Ok(())
}
