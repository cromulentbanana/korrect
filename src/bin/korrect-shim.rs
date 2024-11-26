use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::{Command as ProcessCommand, Stdio};

use anyhow::{anyhow, Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;
use reqwest::blocking::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};

struct KorrectShimConfig {
    korrect_config_path: PathBuf,
    korrect_cache_path: PathBuf,
    korrect_bin_path: PathBuf,
    dl_url: String,
    os: String,
    cpu_arch: String,
    debug: bool,
}

impl KorrectShimConfig {
    fn new(debug: bool) -> Result<Self> {
        let dl_url = env::var("KORRECT_BASE_URL").unwrap_or("https://dl.k8s.io".to_owned());
        let home_dir = dirs::home_dir().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
        })?;
        let config_dir: PathBuf = dirs::config_dir().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Config directory not found")
        })?;
        let cache_dir: PathBuf = dirs::cache_dir().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotFound, "Cache directory not found")
        })?;
        let korrect_config_path = config_dir.join("korrect");
        let korrect_cache_path = cache_dir.join("korrect");
        let korrect_bin_path = home_dir.join(".korrect").join("bin");

        fs::create_dir_all(&korrect_cache_path).ok();
        fs::create_dir_all(&korrect_bin_path).ok();

        let os = detect_os();
        let cpu_arch = detect_cpu_arch();

        Ok(Self {
            korrect_config_path,
            korrect_cache_path,
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

        if self.debug {
            println!(
                "cache_file for kubeconfig [{}] is [{:#?}].",
                &kubeconfig,
                &cache_file.to_str()
            );
        }

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
        version = normalize_version(&version)?;

        // Cache the version
        fs::write(&cache_file, &version)?;

        Ok(version)
    }

    fn get_version_cache_file(&self, kubeconfig: &str) -> Result<PathBuf> {
        let mut hasher = Sha256::new();
        // let contents = fs::read_to_string(&kubeconfig).unwrap_or(kubeconfig.to_owned());
        let contents = fs::read_to_string(kubeconfig).unwrap_or("".to_owned());
        if self.debug {
            println!("contents [{}]", contents);
        }
        hasher.update(contents.as_bytes());
        let hash = format!("{:x}", hasher.finalize())[..5].to_string();
        Ok(self.korrect_cache_path.join(hash))
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

        download_file_with_progress(&url, &target_path).context("Failed to download file")?;

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
        fs::set_permissions(output_path, perms)?;
    }
    Ok(())
}

fn normalize_version(version: &str) -> Result<String> {
    // Define a regex to match the `vX.Y.Z` pattern
    let re = Regex::new(r"v(\d+)\.(\d+)\.(\d+)")?;

    // Search for the pattern in the input string
    if let Some(captures) = re.captures(version) {
        // Construct the normalized version string
        Ok(format!(
            "v{}.{}.{}",
            &captures[1], // X
            &captures[2], // Y
            &captures[3]  // Z
        ))
    } else {
        // Return an error if no match is found
        Err(anyhow!(
            "Version string does not match the expected pattern"
        ))
    }
}

fn main() -> Result<()> {
    let debug = env::var("DEBUG").map_or(false, |v| v == "true");
    let config = KorrectShimConfig::new(debug)?;
    config.run()
}

#[cfg(test)]
mod korrect_shim_tests {
    use super::*;
    use std::env;
    use std::fs;
    

    
    use tempfile::TempDir;

    // Helper function to create a temporary home directory
    fn setup_temp_home() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_str().unwrap().to_owned();
        env::set_var("HOME", &temp_home);
        (temp_dir, temp_home)
    }

    fn remove_temp_home(dir: TempDir) {
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_detect_os() {
        let os = detect_os();
        match env::consts::OS {
            "macos" => assert_eq!(os, "darwin"),
            "windows" => assert_eq!(os, "windows"),
            _ => assert_eq!(os, "linux"),
        }
    }

    #[test]
    fn test_detect_cpu_arch() {
        let arch = detect_cpu_arch();
        match env::consts::ARCH {
            "x86" => assert_eq!(arch, "386"),
            "x86_64" => assert_eq!(arch, "amd64"),
            "arm" => assert_eq!(arch, "arm"),
            "aarch64" => assert_eq!(arch, "arm64"),
            _ => assert_eq!(arch, env::consts::ARCH),
        }
    }

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("v1.2.3").unwrap(), "v1.2.3");
        assert_eq!(normalize_version("v7.24.31").unwrap(), "v7.24.31");
        assert_eq!(
            normalize_version("somethingv1.2.3-alpha").unwrap(),
            "v1.2.3"
        );
        assert_eq!(normalize_version("v1.2.3-alpha").unwrap(), "v1.2.3");

        // Cases that should fail to match and return an error
        assert!(normalize_version("version1.2.3").is_err());
        assert!(normalize_version("invalid").is_err());
    }

    #[test]
    fn test_get_version_cache_file() {
        let (temp_dir, _) = setup_temp_home();
        let config = KorrectShimConfig::new(false).unwrap();

        // Create a temporary kubeconfig file
        let temp_kubeconfig_dir = TempDir::new().unwrap().path().join("");
        let temp_kubeconfig = temp_kubeconfig_dir.join("config");

        fs::create_dir_all(&temp_kubeconfig_dir).unwrap();
        fs::write(&temp_kubeconfig, "test-content").unwrap();

        let cache_file = config
            .get_version_cache_file(temp_kubeconfig.to_str().unwrap())
            .unwrap();
        assert!(cache_file.starts_with(config.korrect_cache_path));

        remove_temp_home(temp_dir);
    }

    #[test]
    fn test_download_kubectl() {
        let (temp_dir, _) = setup_temp_home();

        let mut server = mockito::Server::new();
        let url = server.url();
        let test_file_content = b"A bunch of bytes";

        server
            .mock("GET", "/test-file")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(test_file_content)
            .create();

        env::set_var("KORRECT_BASE_URL", url);
        let config = KorrectShimConfig::new(false).unwrap();

        // Test downloading a specific version
        let version = "v1.23.0";

        let result = config.download_kubectl(version);
        assert!(result.is_ok(), "Test failed: result is {:?}", result);

        let target_path = config.korrect_bin_path.join(format!("kubectl-{}", version));
        assert!(target_path.exists());

        remove_temp_home(temp_dir);
    }

    #[test]
    fn test_get_current_stable_version() {
        let (temp_dir, _) = setup_temp_home();
        let config = KorrectShimConfig::new(false).unwrap();

        let version = config.get_current_stable_version();
        assert!(version.is_ok());
        let version_str = version.unwrap();
        assert!(version_str.starts_with('v'));
        assert!(Regex::new(r"v\d+\.\d+\.\d+")
            .unwrap()
            .is_match(&version_str));

        remove_temp_home(temp_dir);
    }

    #[test]
    fn test_get_server_version_with_cache() {
        let (temp_dir, _) = setup_temp_home();
        let config = KorrectShimConfig::new(false).unwrap();

        // Create a cached version
        let cache_file = config.get_version_cache_file("test-config").unwrap();
        fs::write(&cache_file, "v1.23.0").unwrap();

        let version = config.get_server_version(Some("test-config")).unwrap();
        assert_eq!(version, "v1.23.0");

        remove_temp_home(temp_dir);
    }

    #[test]
    fn test_download_file_with_progress() {
        let mut server = mockito::Server::new();
        let url = server.url();
        let test_file_content = b"v1.3.0";

        server
            .mock("GET", "/test-file")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_header("x-api-key", "1234")
            .with_body(test_file_content)
            .create();

        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("test-file");

        let url = format!("{url}/test-file");
        let result = download_file_with_progress(&url, &output_path);

        assert!(result.is_ok());
        assert!(output_path.exists());
        assert_eq!(std::fs::read(&output_path).unwrap(), test_file_content);
    }
}
