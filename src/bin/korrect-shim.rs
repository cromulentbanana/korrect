use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command as ProcessCommand, Stdio};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};

struct KorrectConfig {
    korrect_path: PathBuf,
    korrect_bin_path: PathBuf,
    dl_url: String,
    os: String,
    cpu_arch: String,
    debug: bool,
}

impl KorrectConfig {
    fn new(debug: bool) -> Result<Self> {
        let home = env::var("HOME")?;
        let dl_url = env::var("KORRECT_BASE_URL").unwrap_or("https://dl.k8s.io".to_owned());
        let korrect_path = Path::new(&home).join(".korrect");
        let korrect_bin_path = korrect_path.join("bin");
        fs::create_dir_all(&korrect_bin_path)?;
        fs::create_dir_all(&korrect_path.join("cache"))?;

        let os = detect_os();
        let cpu_arch = detect_cpu_arch();

        Ok(Self {
            korrect_path,
            korrect_bin_path,
            os,
            cpu_arch,
            debug,
            dl_url,
        })
    }
    fn get_current_stable_version(&self) -> Result<String> {
        let resp = reqwest::blocking::get(format!("{}/release/stable.txt", self.dl_url))?;
        resp.text().map_err(|e| anyhow::anyhow!(e))
    }

    fn get_server_version(&self, kubeconfig: Option<&str>) -> Result<String> {
        let kubeconfig = match kubeconfig {
            Some(config) => config.to_string(),
            //FIXME use a proper home
            None => "~/.kube/config".to_owned(),
        };

        let cache_file = self.get_version_cache_file(&kubeconfig)?;

        // Try reading from cache first
        if let Ok(cached_version) = fs::read_to_string(&cache_file) {
            return Ok(cached_version.trim().to_string());
        }

        // Fetch known version if no cached version
        let current_stable_version = self.get_current_stable_version()?;
        let local_kubectl = self
            .korrect_bin_path
            .join(format!("kubectl-{}", current_stable_version));

        let output = ProcessCommand::new(local_kubectl)
            .arg("version")
            .arg("-o")
            .arg("json")
            .output()?;

        let json: Value = serde_json::from_slice(&output.stdout)?;
        let mut version = match json["serverVersion"]["gitVersion"].as_str() {
            Some(value) => value.to_string(),
            None => {
                return Ok(current_stable_version);
            }
        };

        // Normalize version
        version = normalize_version(&version);

        // Cache the version
        fs::write(&cache_file, &version)?;

        Ok(version)
    }

    fn get_version_cache_file(&self, kubeconfig: &str) -> Result<PathBuf> {
        let mut hasher = Sha256::new();
        hasher.update(kubeconfig.as_bytes());
        let hash = format!("{:x}", hasher.finalize())[..5].to_string();
        Ok(self.korrect_path.join("cache").join(hash))
    }

    fn download_kubectl(&self, version: &str) -> Result<PathBuf> {
        let target_path = self.korrect_bin_path.join(format!("kubectl-{}", version));

        if target_path.exists() {
            return Ok(target_path);
        }

        let url = format!(
            "{}/release/{}/bin/{}/{}/kubectl",
            self.dl_url, version, self.os, self.cpu_arch
        );

        // let resp = reqwest::blocking::get(&url)?;
        // let mut dest = File::create(&target_path)?;
        // dest.write_all(&resp.bytes()?)?;

        download_file_with_progress(&url, &target_path).context("Failed to download file")?;
        // Make executable

        Ok(target_path)
    }

    fn run(&self) -> Result<()> {
        if self.debug {
            println!("Enabled verbose logging.");
        }

        // Ensure the latest stable kubectl is downloaded
        let known_version = self.get_current_stable_version()?;
        let _known_kubectl = self.download_kubectl(&known_version)?;

        // Get server version
        //TODO Fix the dependency on env var KUBECONFIG
        let kconf_owned = std::env::var("KUBECONFIG").ok();
        let kconf = kconf_owned.as_deref();
        let target_version = self.get_server_version(kconf)?;

        // Download target version
        let target_kubectl = self.download_kubectl(&target_version)?;

        if self.debug {
            println!("using [{}].", target_version);
        }

        // Execute kubectl with all arguments
        let status = ProcessCommand::new(target_kubectl)
            .args(env::args().skip(1))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        std::process::exit(status.code().unwrap_or(1));
    }
}

fn detect_os() -> String {
    match env::consts::OS {
        "macos" => "darwin".to_string(),
        "windows" => "windows".to_string(),
        _ => "linux".to_string(),
    }
}

fn detect_cpu_arch() -> String {
    match env::consts::ARCH {
        "x86" => "386".to_string(),
        "x86_64" => "amd64".to_string(),
        "arm" => "arm".to_string(),
        "aarch64" => "arm64".to_string(),
        _ => env::consts::ARCH.to_string(),
    }
}

fn download_file_with_progress(url: &str, output_path: &PathBuf) -> Result<()> {
    // Create a blocking reqwest client
    let client = Client::new();

    // Send a GET request and get the response
    let mut response = client.get(url).send()?;

    // Get the total file size
    let total_size = response.content_length().unwrap_or(0);

    // Create a progress bar
    let pb = ProgressBar::new(total_size);
    // pb.set_style(ProgressStyle::default_spinner());
    pb.set_style(ProgressStyle::default_bar().template("{msg} {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
    .progress_chars("#>-"));
    pb.set_message(format!("Downloading {}", &url));

    // Create the output file
    let mut dest = File::create(output_path)?;

    // Buffer for reading chunks
    let mut buffer = vec![0; 8192]; // 8KB chunks
    let mut downloaded: u64 = 0;

    // Download with progress tracking
    loop {
        let bytes_read = response.read(&mut buffer)?;

        if bytes_read == 0 {
            break;
        }

        dest.write_all(&buffer[0..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }

    // Complete the progress bar
    // pb.finish_with_message("Download complete");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = dest.metadata()?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&output_path, perms)?;
    }
    Ok(())
}

fn normalize_version(version: &str) -> String {
    // Define a regex to match the `vX.Y.Z` pattern
    let re = Regex::new(r"v(\d+)\.(\d+)\.(\d+)").unwrap();

    // Search for the pattern in the input string
    if let Some(captures) = re.captures(version) {
        // Construct the normalized version string
        format!(
            "v{}.{}.{}",
            &captures[1], // X
            &captures[2], // Y
            &captures[3]  // Z
        )
    } else {
        // Return an empty string or handle errors gracefully if no match is found
        String::new()
    }
}

fn main() -> Result<()> {
    let debug = env::var("DEBUG").map_or(false, |v| v == "true");
    let config = KorrectConfig::new(debug)?;
    config.run()
}
