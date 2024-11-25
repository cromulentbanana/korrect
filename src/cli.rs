use clap::{builder::styling, ArgGroup, CommandFactory, Parser, Subcommand};
use clap_complete::{generate, Shell};
use clap_complete_nushell::Nushell;
use std::io::{stdout, Error};

#[derive(clap::ValueEnum, Clone, Copy)]
pub enum ShellType {
    Bash,
    Elvish,
    Fish,
    Nushell,
    Powershell,
    Zsh,
}

impl std::fmt::Display for ShellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShellType::Bash => write!(f, "bash"),
            ShellType::Zsh => write!(f, "zsh"),
            ShellType::Fish => write!(f, "fish"),
            ShellType::Elvish => write!(f, "elvish"),
            ShellType::Nushell => write!(f, "nushell"),
            ShellType::Powershell => write!(f, "powershell"),
        }
    }
}

#[derive(Parser)]
#[command(
    author,
    version = concat!(env!("CARGO_PKG_VERSION"), concat!("_", env!("GIT_SHORT_HASH"))),
    about,
    styles = styles(),
    color = clap::ColorChoice::Auto,
    long_about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Clone, Copy)]
pub enum Commands {
    #[clap(about = "Generates shell completions")]
    #[command(arg_required_else_help = true)]
    Completions {
        #[arg(value_enum)]
        #[arg(help = "Generates shell completion scripts")]
        shell: Option<ShellType>,

        #[arg(long, short, hide = true)]
        help: bool,
    },
    #[clap(group(ArgGroup::new("exclusive_flags")
        .args(&["force", "uninstall"])
        .required(false)))]
    #[clap(about = "Installs the korrect-shim and creates the cache")]
    Setup {
        #[clap(long, default_value = "false")]
        #[clap(help = "Automatically download versions of kubectl when needed")]
        auto_download: bool,
        #[clap(long, default_value = "false", group = "exclusive_flags")]
        #[clap(help = "Overwrite existing korrect installed files")]
        force: bool,
        #[clap(long, default_value = "false", group = "exclusive_flags")]
        #[clap(help = "Remove all korrect installed files")]
        uninstall: bool,
    },
    #[clap(about = "Lists the installed components")]
    List,
}

pub fn generate_completions(shell: Option<ShellType>, help: bool) -> Result<(), Error> {
    let mut cmd = Cli::command();
    let bin_name = cmd.get_name().to_string();
    match shell {
        Some(shell_type) => {
            if help {
                print_shell_specific_instructions(&bin_name, shell_type);
            } else {
                match shell_type {
                    ShellType::Bash => generate(Shell::Bash, &mut cmd, "korrect", &mut stdout()),
                    ShellType::Zsh => generate(Shell::Zsh, &mut cmd, "korrect", &mut stdout()),
                    ShellType::Fish => generate(Shell::Fish, &mut cmd, "korrect", &mut stdout()),
                    ShellType::Powershell => {
                        generate(Shell::PowerShell, &mut cmd, "korrect", &mut stdout())
                    }
                    ShellType::Elvish => {
                        generate(Shell::Elvish, &mut cmd, "korrect", &mut stdout())
                    }
                    ShellType::Nushell => generate(Nushell, &mut cmd, "korrect", &mut stdout()),
                }
            }
        }
        None => {
            cmd.render_help();
            cmd.find_subcommand_mut("completions")
                .unwrap()
                .print_help()?;
        }
    }

    Ok(())
}

fn print_shell_specific_instructions(bin_name: &str, shell: ShellType) {
    println!(
        "\nTo install completions for {}, follow these steps:\n",
        shell
    );

    match shell {
        ShellType::Bash => {
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
        }
        ShellType::Zsh => {
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
        }
        ShellType::Fish => {
            println!("  # Create completions directory");
            println!("  mkdir -p ~/.config/fish/completions");
            println!("  # Generate and save completions");
            println!(
                "  {} completions fish > ~/.config/fish/completions/{}.fish",
                bin_name, bin_name
            );
            println!("  # Completions will be automatically loaded on next shell start");
        }
        ShellType::Elvish => {
            println!("Elvish:");
            println!("  # Create completions directory");
            println!("  mkdir -p ~/.elvish/lib");
            println!("  # Generate and save completions");
            println!(
                "  {} completions elvish > ~/.elvish/lib/{}-completions.elv",
                bin_name, bin_name
            );
            println!("  # Add to ~/.elvish/rc.elv (if not already present)");
            println!("  echo 'use {}-completions' >> ~/.elvish/rc.elv", bin_name);
            println!("  # Restart Elvish or reload configuration");
        }
        _ => {
            println!("Shell-specific instructions not available for this shell.");
        }
    }
}

fn styles() -> styling::Styles {
    styling::Styles::plain()
        .header(
            styling::AnsiColor::BrightBlue.on_default()
                | styling::Effects::BOLD
                | styling::Effects::ITALIC,
        )
        .usage(
            styling::AnsiColor::BrightBlue.on_default()
                | styling::Effects::BOLD
                | styling::Effects::ITALIC,
        )
        .literal(styling::AnsiColor::Green.on_default() | styling::Effects::BOLD)
        .placeholder(styling::AnsiColor::BrightBlue.on_default())
}
