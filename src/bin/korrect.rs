use anyhow::Context;
use clap::{CommandFactory, Parser};
use korrect::cli::{generate_completions, Cli, Commands};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::{env, fs};

fn setup() -> anyhow::Result<()> {
    let home = env::var("HOME").context("Could not find home directory")?;
    let korrect_dir = Path::new(&home).join(".korrect");
    let bin_dir = korrect_dir.join("bin");

    // Create directories if they don't exist
    fs::create_dir_all(&bin_dir)?;

    // Get the path to the current executable
    let current_exe = std::env::current_exe()?;
    let current_dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Could not get parent directory"))?;

    // Copy korrect-shim to ~/.korrect/bin
    let shim_source = current_dir.join("korrect-shim");
    let shim_dest = bin_dir.join("kubectl-shim");
    fs::copy(&shim_source, &shim_dest)?;

    std::os::unix::fs::symlink(&shim_dest, bin_dir.join("kubectl"))?;
    std::os::unix::fs::symlink(&shim_dest, bin_dir.join("k"))?;

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

    // match cli.command {
    //     Some(Commands::Setup) => setup()?,
    //     Some(Commands::List) => list()?,
    //     Some(Commands::Completions) => {
    //         println!("Shell completions not yet implemented");
    //     }
    //     None => {
    //         Cli::command().print_help()?;
    //         println!();
    //     }
    // }

    match cli.command {
        Some(Commands::Completions { shell }) => {
            generate_completions(shell)?;
        }
        Some(Commands::Setup) => {
            // Handle setup command
            setup()?;
        }
        Some(Commands::List) => {
            // Handle list command
            list()?;
        }
        _ => {
            Cli::command().print_help()?;
            println!();
            // Continue with normal kubectl wrapper functionality
        }
    }

    Ok(())
}
