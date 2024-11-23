use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use std::io::{stdout, Error};

#[derive(Parser)]
#[command(
    author,
    version = concat!(env!("CARGO_PKG_VERSION"), concat!("_", env!("GIT_SHORT_HASH"))),
    about,
    color = clap::ColorChoice::Auto,
    long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

// Modify your Commands enum to add completions support:
#[derive(Subcommand)]
pub enum Commands {
    /// Setup korrect-shim in ~/.korrect/bin
    Setup,
    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        #[arg(help = "Shell to generate completions for")]
        shell: Option<Shell>,
    },
    /// List installed components
    List,
}

pub fn generate_completions(shell: Option<Shell>) -> Result<(), Error> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    match shell {
        Some(Shell::Bash) => {
            generate(
                clap_complete::Shell::Bash,
                &mut cmd,
                "korrect",
                &mut stdout(),
            );
        }
        Some(Shell::Zsh) => {
            generate(
                clap_complete::Shell::Zsh,
                &mut cmd,
                "korrect",
                &mut stdout(),
            );
        }
        Some(Shell::Elvish) => {
            generate(
                clap_complete::Shell::Elvish,
                &mut cmd,
                "korrect",
                &mut stdout(),
            );
        }
        Some(Shell::Fish) => {
            generate(
                clap_complete::Shell::Fish,
                &mut cmd,
                "korrect",
                &mut stdout(),
            );
        }
        _ => {
            println!("Shell not recognized {:?}", shell);
            println!("\nTo install completions:");
            print_installation_instructions(&bin_name);
        }
    }

    Ok(())
}

fn print_installation_instructions(bin_name: &str) {
    println!("\nTo install completions, run one of the following commands based on your shell:\n");

    println!("Bash:");
    println!("  # Create completions directory");
    println!("  mkdir -p ~/.bash_completion.d");
    println!("  # Generate and save completions");
    println!(
        "  {} completions bash > ~/.bash_completion.d/{}.bash",
        bin_name, bin_name
    );
    println!("  # Add to ~/.bashrc (if not already present)");
    println!(
        "  echo 'source ~/.bash_completion.d/{}.bash' >> ~/.bashrc",
        bin_name
    );
    println!("  # Reload shell configuration");
    println!("  source ~/.bashrc");

    println!("\nZsh:");
    println!("  # Create completions directory");
    println!("  mkdir -p ~/.zsh/completion");
    println!("  # Generate and save completions");
    println!(
        "  {} completions zsh > ~/.zsh/completion/_{}",
        bin_name, bin_name
    );
    println!("  # Add to ~/.zshrc (if not already present)");
    println!("  echo 'fpath=(~/.zsh/completion $fpath)' >> ~/.zshrc");
    println!("  echo 'autoload -U compinit; compinit' >> ~/.zshrc");
    println!("  # Reload shell configuration");
    println!("  source ~/.zshrc");

    println!("\nNushell:");
    println!("  # Create completions directory");
    println!("  mkdir -p ~/.config/nushell/completions");
    println!("  # Generate and save completions");
    println!(
        "  {} completions nu > ~/.config/nushell/completions/{}.nu",
        bin_name, bin_name
    );
    println!("  # Add to your Nushell config (if not already present)");
    println!(
        "  echo 'source ~/.config/nushell/completions/{}.nu' >> ~/.config/nushell/config.nu",
        bin_name
    );
    println!("  # Restart Nushell or reload configuration");

    println!("\nNote: You can also generate completions to stdout and redirect them wherever you prefer:");
    println!("  {} completions <shell>", bin_name);
}
