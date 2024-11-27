use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{env, fs};

use anyhow::Result;
use clap::{CommandFactory, Parser};

use korrect::cli::{generate_completions, Cli, Commands};

struct Korrect {
    korrect_config_path: PathBuf,
    korrect_cache_path: PathBuf,
    korrect_base_path: PathBuf,
    korrect_bin_path: PathBuf,
    dl_url: String,
}

impl Korrect {
    fn new() -> Result<Self> {
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
        let korrect_base_path = home_dir.join(".korrect");

        Ok(Self {
            korrect_config_path,
            korrect_cache_path,
            korrect_base_path,
            korrect_bin_path,
            dl_url,
        })
    }

    fn setup(&self, auto_download: bool, force: bool, uninstall: bool) -> anyhow::Result<()> {
        let korrect_dirs = vec![
            &self.korrect_base_path,
            &self.korrect_bin_path,
            &self.korrect_cache_path,
            &self.korrect_config_path,
        ];

        if uninstall {
            remove_korrect_directories(korrect_dirs);
            return Ok(());
        }

        create_korrect_directories(korrect_dirs, force);

        // Get the path to the current executable
        let current_exe = std::env::current_exe()?;
        let current_dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Could not get parent directory"))?;

        // Copy korrect-shim to ~/.korrect/bin
        let shim_source = current_dir.join("korrect-shim");
        let shim_dest = self.korrect_bin_path.join("kubectl-shim");

        fs::copy(&shim_source, &shim_dest)?;
        let _ = std::os::unix::fs::symlink(&shim_dest, self.korrect_bin_path.join("kubectl"));
        let _ = std::os::unix::fs::symlink(&shim_dest, self.korrect_bin_path.join("k"));

        // Set executable permissions (rwxr-xr-x)
        let mut perms = fs::metadata(&shim_dest)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&shim_dest, perms)?;

        if auto_download {
            println!("Auto-downloading latest version...");
            // TODO: write config to the shim config file so that it will automatically download
        }

        println!(
            "Successfully installed korrect-shim to {}",
            shim_dest.display()
        );
        println!("Please add {:?} to your PATH", &self.korrect_bin_path);
        println!("export PATH={:?}:$PATH", &self.korrect_bin_path);

        Ok(())
    }

    fn list(&self) -> anyhow::Result<()> {
        if !self.korrect_bin_path.exists() {
            println!("korrect is not set up. Run 'korrect setup' first.");
            return Ok(());
        }

        println!(
            "Installed components in {}:",
            self.korrect_bin_path.display()
        );
        for entry in fs::read_dir(&self.korrect_bin_path)? {
            let entry = entry?;
            println!("- {}", entry.file_name().to_string_lossy());
        }

        Ok(())
    }
}

fn create_korrect_directories(korrect_dirs: Vec<&PathBuf>, force: bool) {
    for dir in korrect_dirs {
        if dir.exists() && force {
            fs::remove_dir_all(dir).ok();
        }
        if !dir.exists() {
            if let Err(e) = fs::create_dir_all(dir) {
                eprintln!("Failed to create directory {:?}: {}", dir, e);
            } else {
                println!("Created directory {:?}", dir);
            }
        } else {
            println!("Directory {:?} already exists", dir);
            println!("Use --force to overwrite.");
        }
    }
}

fn remove_korrect_directories(korrect_dirs: Vec<&PathBuf>) {
    for dir in korrect_dirs {
        if dir.exists() {
            if let Err(e) = fs::remove_dir_all(dir) {
                eprintln!("Failed to remove directory {:?}: {}", dir, e);
            } else {
                println!("Removed directory {:?}", dir);
            }
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let korrect = Korrect::new().unwrap();
    match cli.command {
        Some(Commands::Completions { shell, help }) => {
            generate_completions(shell, help)?;
        }
        Some(Commands::Setup {
            auto_download,
            force,
            uninstall,
        }) => {
            korrect.setup(auto_download, force, uninstall)?;
        }
        Some(Commands::List) => {
            // Handle list command
            korrect.list()?;
        }
        _ => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}

#[cfg(test)]
mod korrect_tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::path::PathBuf;

    use tempfile::TempDir;

    // Helper function to create a temporary home directory
    fn setup_temp_home() -> (TempDir, String) {
        let temp_dir = TempDir::new().unwrap();
        let temp_home = temp_dir.path().to_str().unwrap().to_owned();
        env::set_var("HOME", &temp_home);
        (temp_dir, temp_home)
    }

    fn remove_temp_dir(dir: TempDir) {
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn test_korrect_setup() {
        let (temp_dir, temp_home) = setup_temp_home();

        let korrect = Korrect::new().unwrap();
        korrect.setup(true, false, false).ok();

        assert_eq!(
            korrect.korrect_bin_path,
            PathBuf::from(&temp_home).join(".korrect").join("bin")
        );
        assert_eq!(
            korrect.korrect_config_path,
            PathBuf::from(&temp_home).join(".config").join("korrect")
        );
        assert_eq!(
            korrect.korrect_cache_path,
            PathBuf::from(&temp_home).join(".cache").join("korrect")
        );
        assert!(korrect.korrect_cache_path.exists());
        assert!(korrect.korrect_config_path.exists());
        assert!(korrect.korrect_bin_path.exists());

        remove_temp_dir(temp_dir);
    }

    #[test]
    fn test_korrect_new_with_custom_base_url() {
        // Set a custom base URL
        env::set_var("KORRECT_BASE_URL", "https://test.example.com");

        let korrect = Korrect::new().unwrap();

        assert_eq!(korrect.dl_url, "https://test.example.com");

        // Clean up
        env::remove_var("KORRECT_BASE_URL");
    }

    #[test]
    fn test_korrect_new_default_base_url() {
        // Ensure no custom URL is set
        env::remove_var("KORRECT_BASE_URL");

        let korrect = Korrect::new().unwrap();

        assert_eq!(korrect.dl_url, "https://dl.k8s.io");
    }

    #[test]
    fn test_korrect_setup_uninstall() {
        let (temp_dir, _) = setup_temp_home();

        // Manually create directories to simulate existing setup
        let korrect = Korrect::new().unwrap();
        fs::create_dir_all(&korrect.korrect_bin_path).unwrap();
        fs::create_dir_all(&korrect.korrect_cache_path).unwrap();
        fs::create_dir_all(&korrect.korrect_config_path).unwrap();

        // Perform uninstall
        korrect.setup(false, false, true).unwrap();

        // Verify directories are removed
        assert!(!korrect.korrect_bin_path.exists());
        assert!(!korrect.korrect_cache_path.exists());
        assert!(!korrect.korrect_config_path.exists());

        remove_temp_dir(temp_dir);
    }

    #[test]
    #[ignore]
    fn test_korrect_setup_force_overwrite() {
        let (temp_dir, _) = setup_temp_home();

        // Create some files in the directories
        let korrect = Korrect::new().unwrap();
        fs::create_dir_all(&korrect.korrect_bin_path).unwrap();
        fs::write(korrect.korrect_bin_path.join("test_file"), "test content").unwrap();

        // Perform setup with force
        korrect.setup(false, true, false).unwrap();

        // Verify directories exist and are clean
        assert!(korrect.korrect_bin_path.exists());
        assert!(!korrect.korrect_bin_path.join("test_file").exists());

        remove_temp_dir(temp_dir);
    }

    #[test]
    #[ignore]
    fn test_list_command_not_setup() {
        let (temp_dir, _temp_home) = setup_temp_home();

        let korrect = Korrect::new().unwrap();
        // Capture stdout
        let mut output = Vec::new();
        {
            let _handle = std::io::Cursor::new(&mut output);
            // Redirect stdout to our buffer
            let result = std::panic::catch_unwind(|| {
                korrect.list().unwrap();
            });

            // Check if the result is as expected
            assert!(result.is_ok());

            remove_temp_dir(temp_dir);
        }

        // Convert output to string
        let output_str = String::from_utf8(output).unwrap();
        assert!(
            output_str.contains("korrect is not set up"),
            "Output was: {}",
            output_str
        );
    }

    #[test]
    #[ignore]
    fn test_list_command_with_existing_bin() {
        let (temp_dir, temp_home) = setup_temp_home();

        // Create .korrect/bin directory with some files
        let bin_dir = PathBuf::from(&temp_home).join(".korrect").join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(bin_dir.join("kubectl"), "test content").unwrap();
        fs::write(bin_dir.join("k"), "test content").unwrap();

        let korrect = Korrect::new().unwrap();
        // Capture stdout
        let mut output = Vec::new();
        {
            let _handle = std::io::Cursor::new(&mut output);
            // Redirect stdout to our buffer
            let result = std::panic::catch_unwind(|| {
                korrect.list().unwrap();
            });

            // Check if the result is as expected
            assert!(result.is_ok());

            remove_temp_dir(temp_dir);
        }

        // Convert output to string
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Installed components"));
        assert!(output_str.contains("kubectl"));
        assert!(output_str.contains("k"));
    }

    #[test]
    fn test_create_korrect_directories() {
        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");

        let dirs = vec![&dir1, &dir2];

        // Create directories
        create_korrect_directories(dirs.clone(), false);

        // Verify directories exist
        assert!(dir1.exists());
        assert!(dir2.exists());

        remove_temp_dir(temp_dir);
    }

    #[test]
    fn test_remove_korrect_directories() {
        let temp_dir = TempDir::new().unwrap();
        let dir1 = temp_dir.path().join("dir1");
        let dir2 = temp_dir.path().join("dir2");

        // Create directories
        fs::create_dir_all(&dir1).unwrap();
        fs::create_dir_all(&dir2).unwrap();

        let dirs = vec![&dir1, &dir2];

        // Remove directories
        remove_korrect_directories(dirs.clone());

        // Verify directories are removed
        assert!(!dir1.exists());
        assert!(!dir2.exists());
    }
}
