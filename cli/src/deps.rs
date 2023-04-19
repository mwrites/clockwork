use {
    crate::config::{
        self,
        CliConfig,
    },
    anyhow::{
        Context,
        Result,
    },
    bzip2::read::BzDecoder,
    clap::crate_version,
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

pub fn download_deps(runtime_dir: &Path) -> Result<()> {
    let solana_tag = env!("GEYSER_INTERFACE_VERSION").to_owned().to_tag_version();
    let clockwork_tag = crate_version!().to_owned().to_tag_version();

    download_and_extract(
        runtime_dir,
        &CliConfig::solana_release_url(&solana_tag),
        &CliConfig::default_runtime_dir().join(CliConfig::solana_release_archive()),
        config::SOLANA_ARCHIVE_PREFIX,
    )?;
    download_and_extract(
        runtime_dir,
        &CliConfig::clockwork_release_url(&clockwork_tag),
        &CliConfig::default_runtime_dir().join(CliConfig::clockwork_release_archive()),
        config::CLOCKWORK_ARCHIVE_PREFIX,
    )
}

pub fn download_and_extract(
    runtime_dir: &Path,
    src_url: &str,
    dest_path: &Path,
    archive_prefix: &str,
) -> Result<()> {
    download_file(src_url, &dest_path)?;
    extract_archive(&dest_path, runtime_dir, archive_prefix)
}

fn download_file(url: &str, dest: &Path) -> Result<()> {
    println!("Downloading {}", url);
    let resp = get(url).context(format!("Failed to download file from {}", url))?;
    if resp.status() != reqwest::StatusCode::OK {
        return Err(anyhow::anyhow!("File not found at {}", url));
    }

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

fn extract_archive(archive_path: &Path, runtime_dir: &Path, strip_prefix: &str) -> Result<()> {
    // create runtime dir if necessary
    fs::create_dir_all(runtime_dir)?;

    let file =
        File::open(&archive_path).context(format!("Failed to open file {:#?}", archive_path))?;
    let mut archive = Archive::new(BzDecoder::new(file));

    // TODO: refactor to onyl extract specific files, and do not rely on prefix

    println!("Extracted the following files:");
    archive
        .entries()?
        .filter_map(|e| e.ok())
        .map(|mut entry| -> Result<PathBuf> {
            let path = entry
                .path()?
                .strip_prefix(strip_prefix)
                .context(format!("Failed to strip prefix {}", strip_prefix))?
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

trait ToTagVersion {
    fn to_tag_version(&self) -> String;
}

impl ToTagVersion for String {
    fn to_tag_version(&self) -> String {
        if !self.starts_with("v") {
            format!("v{}", self)
        } else {
            self.to_owned()
        }
    }
}
