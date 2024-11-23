use anyhow::Context;
use clap::{CommandFactory, Parser};
use korrect::cli::{generate_completions, Cli, Commands};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{env, fs};

fn setup(auto_download: bool, force: bool, uninstall: bool) -> anyhow::Result<()> {
    let home = env::var("HOME").context("Could not find home directory")?;
    let korrect_dir = Path::new(&home).join(".korrect");
    let bin_dir = korrect_dir.join("bin");

    if uninstall {
        if korrect_dir.exists() {
            fs::remove_dir_all(&korrect_dir)?;
            println!(
                "Successfully uninstalled korrect from {}",
                bin_dir.display()
            );
        } else {
            println!(
                "Nothing to uninstall - {} does not exist",
                bin_dir.display()
            );
        }
        return Ok(());
    }

    // Check if directory exists and force flag is not set
    if bin_dir.exists() && !force {
        println!("Installation directory already exists. Use --force to overwrite.");
        return Ok(());
    }

    // Create directories if they don't exist
    fs::create_dir_all(&bin_dir)?;

    // Get the path to the current executable
    let current_exe = std::env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not get parent directory"))?;

    if auto_download {
        println!("Auto-downloading latest version...");
        // TODO: write config to the shim config file so that it will automatically download
    }

    // Copy korrect-shim to ~/.korrect/bin
    let shim_source = current_dir.join("korrect-shim");
    let shim_dest = bin_dir.join("kubectl-shim");

    // Remove existing files if force is true
    if force {
        let _ = fs::remove_file(&shim_dest);
        let _ = fs::remove_file(bin_dir.join("kubectl"));
        let _ = fs::remove_file(bin_dir.join("k"));
    }

    fs::copy(&shim_source, &shim_dest)?;
    let _ = std::os::unix::fs::symlink(&shim_dest, bin_dir.join("kubectl"));
    let _ = std::os::unix::fs::symlink(&shim_dest, bin_dir.join("k"));

    // Set executable permissions (rwxr-xr-x)
    let mut perms = fs::metadata(&shim_dest)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&shim_dest, perms)?;

    println!(
        "Successfully installed korrect-shim to {}",
        shim_dest.display()
    );
    println!("Please add {:?} to your PATH", bin_dir);
    println!("export PATH={:?}:$PATH", bin_dir);
    Ok(())
}

fn list() -> anyhow::Result<()> {
    let home = env::var("HOME").context("Could not find home directory")?;
    let bin_dir = Path::new(&home).join(".korrect").join("bin");

    if !bin_dir.exists() {
        println!("korrect is not set up. Run 'korrect setup' first.");
        return Ok(());
    }

    println!("Installed components in {}:", bin_dir.display());
    for entry in fs::read_dir(bin_dir)? {
        let entry = entry?;
        println!("- {}", entry.file_name().to_string_lossy());
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Completions { shell }) => {
            generate_completions(shell)?;
        }
        Some(Commands::Setup {
            auto_download,
            force,
            uninstall,
        }) => {
            setup(auto_download, force, uninstall)?;
        }
        Some(Commands::List) => {
            // Handle list command
            list()?;
        }
        _ => {
            Cli::command().print_help()?;
            println!();
        }
    }

    Ok(())
}
