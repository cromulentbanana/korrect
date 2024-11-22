use regex::Regex;
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use reqwest;
use serde_json::Value;
use sha2::{Digest, Sha256};

struct KxConfig {
    kx_path: PathBuf,
    os: String,
    cpu_arch: String,
    debug: bool,
}

impl KxConfig {
    fn new() -> Result<Self> {
        let home = env::var("HOME")?;
        let kx_path = Path::new(&home).join(".kx");
        fs::create_dir_all(kx_path.join("cache"))?;

        let os = detect_os();
        let cpu_arch = detect_cpu_arch();
        let debug = env::var("DEBUG").map_or(false, |v| v == "true");

        Ok(Self {
            kx_path,
            os,
            cpu_arch,
            debug,
        })
    }

    fn get_known_version() -> Result<String> {
        let resp = reqwest::blocking::get("https://dl.k8s.io/release/stable.txt")?;
        resp.text().map_err(|e| anyhow::anyhow!(e))
    }

    fn get_server_version(&self) -> Result<String> {
        let kubeconfig = env::var("KUBECONFIG")?;
        let cache_file = self.get_version_cache_file(&kubeconfig)?;

        // Try reading from cache first
        if let Ok(cached_version) = fs::read_to_string(&cache_file) {
            return Ok(cached_version.trim().to_string());
        }

        // Fetch known version if no cached version
        let known_version = Self::get_known_version()?;
        let local_kubectl = self.kx_path.join(format!("kubectl-{}", known_version));

        let output = Command::new(local_kubectl)
            .arg("version")
            .arg("-o")
            .arg("json")
            .output()?;

        let json: Value = serde_json::from_slice(&output.stdout)?;
        let mut version = json["serverVersion"]["gitVersion"]
            .as_str()
            .context("Unable to get version information from cluster")?
            .to_string();

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
        Ok(self.kx_path.join("cache").join(hash))
    }

    fn download_kubectl(&self, version: &str) -> Result<PathBuf> {
        let target_path = self.kx_path.join(format!("kubectl-{}", version));

        if target_path.exists() {
            return Ok(target_path);
        }

        let url = format!(
            "https://dl.k8s.io/release/{}/bin/{}/{}/kubectl",
            version, self.os, self.cpu_arch
        );

        let resp = reqwest::blocking::get(&url)?;
        let mut dest = File::create(&target_path)?;
        dest.write_all(&resp.bytes()?)?;

        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = dest.metadata()?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&target_path, perms)?;
        }

        Ok(target_path)
    }

    fn run(&self) -> Result<()> {
        if self.debug {
            println!("Enabled verbose logging.");
        }

        // Ensure the latest stable kubectl is downloaded
        let known_version = Self::get_known_version()?;
        let _known_kubectl = self.download_kubectl(&known_version)?;

        // Get server version
        let target_version = self.get_server_version()?;

        // Download target version
        let target_kubectl = self.download_kubectl(&target_version)?;

        if self.debug {
            println!("using [{}].", target_version);
        }
        // Execute kubectl with all arguments
        let status = Command::new(target_kubectl)
            .args(env::args().skip(1))
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()?;

        std::process::exit(status.code().unwrap_or(1));
    }
}

fn detect_os() -> String {
    println!("detect_os");
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

fn normalize_version(version: &str) -> String {
    println!("normalize vesion");
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
    let config = KxConfig::new()?;
    config.run()
}
